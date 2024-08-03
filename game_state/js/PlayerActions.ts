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
    document.addEventListener("keydown", this.onKeyDown.bind(this));
  }

  onKeyDown(event: KeyboardEvent) {
    if (event.key === "c") {
      console.log("Creating ship");
      const intersection = this.waterIntersection();
      if (!intersection) return;
      const { x, y } = intersection.point;
      this.shipsManager.createShip(x, y);
    }
  }

  onMouseMove(event: MouseEvent) {
    this.mouse.x = event.offsetX;
    this.mouse.y = event.offsetY;
  }

  private pointerdown(event: PointerEvent) {
    this.mouse.x = event.offsetX;
    this.mouse.y = event.offsetY;

    const hasShift = event.shiftKey;
    if (hasShift) {
      const boat = this.boatClicked();
      if (boat !== null) {
        this.shipsManager.selectBoat(boat);
      }
      return;
    }

    const intersection = this.waterIntersection();
    if (!intersection) {
      return;
    }
    const { x, y } = intersection.point;
    if (event.button === 2) {
      this.shipsManager.moveSelected(x, y);
    } else {
      this.shipsManager.shoot(x, y);
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
