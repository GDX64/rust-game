import * as THREE from "three";
import { ShipsManager } from "./ShipsManager";
import { CameraControl } from "./CameraControl";
import { Water } from "./Water";
import { Terrain } from "./Terrain";
import { LeaderBoards } from "./LeaderBoards";
import { config } from "../config/Config";

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
  readonly selectionDiv = document.createElement("div");
  axesHelper: THREE.AxesHelper;

  constructor(
    public canvas: HTMLCanvasElement,
    public shipsManager: ShipsManager,
    public camera: CameraControl,
    public water: Water,
    public terrain: Terrain,
    public leaderBoards: LeaderBoards
  ) {
    this.mouse = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
    this.selectionDiv.style.position = "absolute";
    this.selectionDiv.style.pointerEvents = "none";
    this.selectionDiv.style.zIndex = "10";
    this.selectionDiv.style.border = "2px solid white";

    this.axesHelper = new THREE.AxesHelper(1000);
    this.shipsManager.scene.add(this.axesHelper);
  }

  showHelper(val: boolean) {
    this.axesHelper.visible = val;
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
    if (config.preventDefaults) {
      window.onbeforeunload = () => {
        return false;
      };
    }

    this.terrain.minimap.mapClick$.subscribe((point) => {
      this.camera.changeCameraPosition(
        new THREE.Vector3(point.x, point.y, this.camera.camera.position.z)
      );
    });

    document.addEventListener("pointerleave", this.pointerleave.bind(this));
    this.canvas.addEventListener("pointerdown", this.pointerdown.bind(this));
    document.addEventListener("pointermove", this.pointermove.bind(this));
    document.addEventListener("pointerup", this.pointerup.bind(this));
    this.canvas.addEventListener("contextmenu", (event) =>
      event.preventDefault()
    );
    document.addEventListener("keydown", this.onKeyDown.bind(this));
    document.addEventListener("keyup", this.onKeyUp.bind(this));
    document.addEventListener("kepress", (event) => {
      if (config.preventDefaults) {
        event.preventDefault();
      }
    });
    document.addEventListener(
      "wheel",
      (event: WheelEvent) => {
        event.preventDefault();
        event.stopPropagation();
        if (event.ctrlKey) {
          const delta = event.deltaY;
          let shootCircle = this.game.shoot_radius() * (1 + delta / 500);
          shootCircle = Math.min(100, Math.max(3, shootCircle));
          this.game.change_shoot_radius(shootCircle);
        } else {
          this.camera.onWeel(event);
        }
      },
      { passive: false }
    );
  }

  appendSelectionRectange(el: HTMLElement) {
    el.appendChild(this.selectionDiv);
  }

  destroy() {
    this.selectionDiv.remove();
  }

  pointerup(event: PointerEvent) {
    this.mouse.x = event.pageX;
    this.mouse.y = event.pageY;
    const selection = this.currentSelection();
    if (selection) {
      const { start, end } = selection;
      const screenStart = this.screenSpacePoint(start);
      const screenEnd = this.screenSpacePoint(end);

      const shipV = new THREE.Vector3();
      this.shipsManager.select((ship) => {
        const { x, y } = ship.position;
        shipV.set(x, y, 0);
        shipV.project(this.camera.camera);
        const xMin = Math.min(screenStart.x, screenEnd.x);
        const xMax = Math.max(screenStart.x, screenEnd.x);
        const yMin = Math.min(screenStart.y, screenEnd.y);
        const yMax = Math.max(screenStart.y, screenEnd.y);
        const xInside = shipV.x > xMin && shipV.x < xMax;
        const yInside = shipV.y < yMax && shipV.y > yMin;
        return xInside && yInside;
      });
      this.changeState(States.IDLE);
    }
  }

  currentSelection() {
    if (this.state !== States.SELECTING) {
      return null;
    }
    const end = this.mouse;
    const start = this.selectionStart;

    if (start && end) {
      const isStartOnTheLeft = start.x < end.x;
      if (isStartOnTheLeft) {
        return { start: start, end: end };
      }
      return { start: end, end: start };
    }
  }

  pointerleave(_event: PointerEvent) {
    this.canvas.style.cursor = "auto";
    this.shipsManager.aimCircle.visible = false;
    this.mouse.x = this.width / 2;
    this.mouse.y = this.height / 2;
    this.changeState(States.IDLE);
  }

  onKeyUp(event: KeyboardEvent) {
    if (config.preventDefaults) {
      event.preventDefault();
    }
    if (event.key === "Control") {
      this.canvas.style.cursor = "auto";
      this.shipsManager.aimCircle.visible = false;
      this.changeState(States.IDLE);
    }
  }

  private changeState(newState: States) {
    this.state = newState;
    if (newState !== States.SELECTING) {
      this.selectionDiv.style.display = "none";
    }
    if (newState === States.SHOOTING) {
      this.leaderBoards.hideLeaderBoards();
      this.terrain.minimap.hideMinimap();
    } else {
      this.terrain.minimap.showMinimap();
      this.leaderBoards.showLeaderBoards();
    }
  }

  onKeyDown(event: KeyboardEvent) {
    if (config.preventDefaults) {
      event.preventDefault();
    }
    if (event.key === "c") {
      const intersection = this.waterIntersection();
      if (!intersection) return;
      const { x, y } = intersection.point;
      this.shipsManager.createShip(x, y);
    }
    if (event.ctrlKey) {
      this.canvas.style.cursor = "crosshair";
      this.shipsManager.aimCircle.visible = true;
      this.changeState(States.SHOOTING);
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
        const canShoot = this.game.can_shoot_here(x, y);
        const margin = this.game.shoot_radius();
        this.shipsManager.aimCircle.position.set(x, y, 0);
        this.shipsManager.aimCircle.scale.set(margin, margin, 1);
        if (canShoot) {
          this.shipsManager.aimCircle.material.color = new THREE.Color(
            "#fbff00"
          );
        } else {
          this.shipsManager.aimCircle.material.color = new THREE.Color(
            "#ff0000"
          );
        }
      }
    }
  }

  private handleSelection() {
    const selection = this.currentSelection();
    if (selection) {
      const { start, end } = selection;

      const width = Math.abs(start.x - end.x);
      const height = Math.abs(start.y - end.y);
      const startX = Math.min(start.x, end.x);
      const startY = Math.min(start.y, end.y);

      this.selectionDiv.style.width = `${width}px`;
      this.selectionDiv.style.height = `${height}px`;
      this.selectionDiv.style.left = `${startX}px`;
      this.selectionDiv.style.top = `${startY}px`;
      this.selectionDiv.style.display = "block";

      // const finalMatrix = selectionRectangle.matrix.clone();
    } else {
      this.selectionDiv.style.display = "none";
    }
  }

  pointermove(event: PointerEvent) {
    this.mouse.x = event.pageX;
    this.mouse.y = event.pageY;
    this.handleSelection();
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
        this.changeState(States.SELECTING);
        this.selectionStart.x = event.offsetX;
        this.selectionStart.y = event.offsetY;
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

  private waterIntersection(point = this.mouse): THREE.Intersection | null {
    this.rayCaster.setFromCamera(
      this.screenSpacePoint(point),
      this.camera.camera
    );
    const intersects = this.water.intersects(this.rayCaster);
    return intersects[0] ?? null;
  }
}
