import * as THREE from "three";
import { ShipsManager } from "./ShipsManager";
import { CameraControl } from "./CameraControl";
import { Water } from "./Water";

export class PlayerActions {
  readonly mouse = new THREE.Vector2(0, 0);
  readonly rayCaster = new THREE.Raycaster();

  constructor(
    public canvas: HTMLCanvasElement,
    public shipsManager: ShipsManager,
    public camera: CameraControl,
    public water: Water
  ) {}

  get width() {
    return this.canvas.offsetWidth;
  }

  get height() {
    return this.canvas.offsetHeight;
  }

  get game() {
    return this.shipsManager.game;
  }

  screenSpaceMouse() {
    return new THREE.Vector2(
      (this.mouse.x / this.width) * 2 - 1,
      -(this.mouse.y / this.height) * 2 + 1
    );
  }

  bindEvents() {
    this.canvas.addEventListener("pointerdown", this.pointerdown.bind(this));
    this.canvas.addEventListener("pointermove", this.onMouseMove.bind(this));
    this.canvas.addEventListener("contextmenu", (event) =>
      event.preventDefault()
    );
    document.addEventListener("keydown", this.onKeyDown.bind(this));
    document.addEventListener("keyup", this.onKeyUp.bind(this));
  }

  onKeyUp(event: KeyboardEvent) {
    this.canvas.style.cursor = "auto";
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
  }

  onMouseMove(event: PointerEvent) {
    this.mouse.x = event.offsetX;
    this.mouse.y = event.offsetY;
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

  private waterIntersection(): THREE.Intersection | null {
    this.rayCaster.setFromCamera(this.screenSpaceMouse(), this.camera.camera);
    const intersects = this.water.intersects(this.rayCaster);
    return intersects[0] ?? null;
  }
}
