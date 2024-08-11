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

  constructor(
    public canvas: HTMLCanvasElement,
    public shipsManager: ShipsManager,
    public camera: CameraControl,
    public water: Water
  ) {
    this.mouse = { x: canvas.offsetWidth / 2, y: canvas.offsetHeight / 2 };

    const axesHelper = new THREE.AxesHelper(1000);
    this.shipsManager.scene.add(axesHelper);
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
      const { start, end, basis } = selection;
      //we need to convert every body to camera perspective
      //so that we can compare it as if it was a normal rectangle
      const invertedBasis = basis.clone().invert();
      const startCamera = start.clone().applyMatrix4(invertedBasis);
      const endCamera = end.clone().applyMatrix4(invertedBasis);
      const startX = Math.min(startCamera.x, endCamera.x);
      const startY = Math.min(startCamera.y, endCamera.y);
      const endX = Math.max(startCamera.x, endCamera.x);
      const endY = Math.max(startCamera.y, endCamera.y);
      this.shipsManager.select((ship) => {
        const shipPos = new THREE.Vector3(
          ship.position[0],
          ship.position[1],
          0
        );
        shipPos.applyMatrix4(invertedBasis);
        return (
          shipPos.x > startX &&
          shipPos.x < endX &&
          shipPos.y > startY &&
          shipPos.y < endY
        );
      });
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
      this.shipsManager.auto_shoot();
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
      const { start, end, basis } = selection;
      this.shipsManager.selectionRectangle.visible = true;
      const diff = end.clone().sub(start);
      //When calculating width and height, we need to do it in the coordinates of the camera
      diff.applyMatrix4(basis.clone().invert());
      const width = diff.x;
      const height = diff.y;
      this.shipsManager.selectionRectangle.scale.set(width, height, 1);
      const quaternion = new THREE.Quaternion();
      selection.basis.decompose(
        new THREE.Vector3(),
        quaternion,
        new THREE.Vector3()
      );
      this.shipsManager.selectionRectangle.setRotationFromQuaternion(
        quaternion
      );
      this.shipsManager.selectionRectangle.position.set(start.x, start.y, 0);
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
