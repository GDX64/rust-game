import * as THREE from "three";
import { ShipsManager } from "./ShipsManager";
import { CameraControl } from "./CameraControl";
import { Water } from "./Water";

enum States {
  IDLE,
  SHOOTING,
  SELECTING,
}

export class PlayerActions {
  readonly mouse;
  readonly rayCaster = new THREE.Raycaster();
  private state = States.IDLE;
  private readonly selectionStart = { x: 0, y: 0 };
  private cameraAxes = new THREE.AxesHelper(1000);

  constructor(
    public canvas: HTMLCanvasElement,
    public shipsManager: ShipsManager,
    public camera: CameraControl,
    public water: Water
  ) {
    this.mouse = { x: canvas.offsetWidth / 2, y: canvas.offsetHeight / 2 };

    const axesHelper = new THREE.AxesHelper(1000);
    this.shipsManager.scene.add(axesHelper);
    this.shipsManager.scene.add(this.cameraAxes);
  }

  get width() {
    return this.canvas.offsetWidth;
  }

  get height() {
    return this.canvas.offsetHeight;
  }

  get game() {
    return this.shipsManager.game;
  }

  screenSpacePoint(point = this.mouse) {
    return new THREE.Vector2(
      (point.x / this.width) * 2 - 1,
      -(point.y / this.height) * 2 + 1
    );
  }

  bindEvents() {
    this.canvas.addEventListener("pointerleave", this.pointerleave.bind(this));
    this.canvas.addEventListener("pointerdown", this.pointerdown.bind(this));
    this.canvas.addEventListener("pointermove", this.pointermove.bind(this));
    this.canvas.addEventListener("pointerup", this.pointerup.bind(this));
    this.canvas.addEventListener("contextmenu", (event) =>
      event.preventDefault()
    );
    document.addEventListener("keydown", this.onKeyDown.bind(this));
    document.addEventListener("keyup", this.onKeyUp.bind(this));
  }

  pointerup(event: PointerEvent) {
    this.mouse.x = event.offsetX;
    this.mouse.y = event.offsetY;
    const selection = this.currentSelection();
    if (selection) {
      // this.shipsManager.selectBoatsInRect(selection.start, selection.end);
      this.state = States.IDLE;
    }
  }

  currentSelection() {
    if (this.state !== States.SELECTING) {
      return null;
    }
    const end = this.waterIntersection(this.mouse);
    const start = this.waterIntersection(this.selectionStart);
    if (start && end) {
      const basis = this.camera.basisMatrix();
      const basisInverse = basis.clone().invert();
      start.point.applyMatrix4(basisInverse);
      end.point.applyMatrix4(basisInverse);
      return { start: start.point, end: end.point, basis };
    }
  }

  pointerleave(_event: PointerEvent) {
    this.canvas.style.cursor = "auto";
    this.shipsManager.aimCircle.visible = false;
    this.mouse.x = this.width / 2;
    this.mouse.y = this.height / 2;
    this.state = States.IDLE;
  }

  onKeyUp(event: KeyboardEvent) {
    if (event.key === "Control") {
      this.canvas.style.cursor = "auto";
      this.shipsManager.aimCircle.visible = false;
      this.state = States.IDLE;
    }
  }

  onKeyDown(event: KeyboardEvent) {
    if (event.key === "c") {
      console.log("Creating ship");
      const intersection = this.waterIntersection();
      if (!intersection) return;
      const { x, y } = intersection.point;
      this.shipsManager.createShip(x, y);
    }
    if (event.ctrlKey) {
      this.canvas.style.cursor = "crosshair";
      this.shipsManager.aimCircle.visible = true;
      this.state = States.SHOOTING;
    }
    if (event.key === "b") {
      const intersection = this.waterIntersection();
      if (!intersection) return;
      const { x, y } = intersection.point;
      this.game.add_bot_ship_at(x, y);
    }
    if (event.key === " ") {
      this.targetSelected();
    }
  }

  targetSelected() {
    const selected = this.shipsManager.selectedShips().next().value;
    if (!selected) {
      return;
    }
    const [x, y] = selected.position;
    this.camera.changeTarget(new THREE.Vector3(x, y, 0));
  }

  handleMousePos() {
    const isCloserToTheRight = this.mouse.x > this.width * 0.95;
    if (isCloserToTheRight) {
      this.camera.rotateAroundZ(-1);
    }
    const isCloserToTheLeft = this.mouse.x < this.width * 0.05;
    if (isCloserToTheLeft) {
      this.camera.rotateAroundZ(1);
    }
    const isCloserToTop = this.mouse.y < this.height * 0.05;
    if (isCloserToTop) {
      this.camera.moveForward(1);
    }
    const isCloserToBottom = this.mouse.y > this.height * 0.95;
    if (isCloserToBottom) {
      this.camera.moveForward(-1);
    }
  }

  tick() {
    this.handleMousePos();
    if (this.state === States.SHOOTING) {
      const intersection = this.waterIntersection();
      if (intersection) {
        const { x, y } = intersection.point;
        const margin = this.game.shoot_error_margin(x, y);
        if (margin) {
          this.shipsManager.aimCircle.position.set(x, y, 0);
          this.shipsManager.aimCircle.scale.set(margin, margin, 1);
        } else {
          this.shipsManager.aimCircle.visible = false;
        }
      }
    }
  }

  pointermove(event: PointerEvent) {
    this.mouse.x = event.offsetX;
    this.mouse.y = event.offsetY;
    const selection = this.currentSelection();
    if (selection) {
      this.shipsManager.selectionRectangle.visible = true;
      const width = selection.end.x - selection.start.x;
      const height = selection.end.y - selection.start.y;
      this.shipsManager.selectionRectangle.scale.set(width, height, 1);
      this.cameraAxes.position.set(-100, 0, 0);
      const quaternion = new THREE.Quaternion();
      selection.basis.decompose(
        new THREE.Vector3(),
        quaternion,
        new THREE.Vector3()
      );
      this.cameraAxes.setRotationFromQuaternion(quaternion);
      this.shipsManager.selectionRectangle.setRotationFromQuaternion(
        quaternion
      );
      const startTransformed = selection.start.applyMatrix4(selection.basis);
      this.shipsManager.selectionRectangle.position.set(
        startTransformed.x,
        startTransformed.y,
        0
      );
      console.log(width.toFixed(), height.toFixed());
    } else {
      this.shipsManager.selectionRectangle.visible = false;
    }
  }

  private pointerdown(event: PointerEvent) {
    event.preventDefault();
    this.mouse.x = event.offsetX;
    this.mouse.y = event.offsetY;

    const hasShift = event.shiftKey;
    const hasControl = event.ctrlKey;

    const intersection = this.waterIntersection();
    if (!intersection) {
      return;
    }
    const { x, y } = intersection.point;
    if (event.button === 2) {
      this.shipsManager.moveSelected(x, y);
    } else if (hasControl) {
      this.shipsManager.shoot(x, y);
    } else {
      const boat = this.boatClicked();
      if (boat == null) {
        if (hasShift) {
          return;
        } else {
          this.shipsManager.clearSelection();
        }
        this.state = States.SELECTING;
        this.selectionStart.x = event.offsetX;
        this.selectionStart.y = event.offsetY;
      } else {
        if (hasShift) {
          this.shipsManager.selectBoat(boat);
        } else {
          this.shipsManager.clearSelection();
          this.shipsManager.selectBoat(boat);
        }
      }
    }
  }

  private boatClicked(): number | null {
    const intersection = this.waterIntersection();
    if (!intersection) return null;
    const { x, y } = intersection.point;
    return this.shipsManager.getBoatAt(x, y);
  }

  private waterIntersection(point = this.mouse): THREE.Intersection | null {
    this.rayCaster.setFromCamera(
      this.screenSpacePoint(point),
      this.camera.camera
    );
    const intersects = this.water.intersects(this.rayCaster);
    return intersects[0] ?? null;
  }
}
