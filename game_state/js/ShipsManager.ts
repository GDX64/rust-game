import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/boat.glb?url";
import { ExplosionData, ExplosionManager } from "./Particles";
import { Water } from "./Water";
import { Subject } from "rxjs";

type ShipData = {
  player_id: number;
  id: number;
  position: [number, number];
  speed: [number, number];
  acceleration: [number, number];
  orientation: [number, number];
};

type Bullet = {
  position: [number, number, number];
  speed: [number, number, number];
  id: number;
  player_id: number;
};

const SHIP_SIZE = 10;

const up = new THREE.Vector3(0, 0, 1);

export class ShipsManager {
  boatMesh: THREE.InstancedMesh = new THREE.InstancedMesh(
    new THREE.BoxGeometry(1, 1),
    new THREE.MeshBasicMaterial(),
    1
  );
  selected: number[] = [];
  outlines;
  private explosionManager: ExplosionManager;
  private bulletModel: THREE.InstancedMesh;
  private ships: ShipData[] = [];
  private arrowHelper = new THREE.ArrowHelper();
  selected$ = new Subject<THREE.InstancedMesh>();
  showArrow = false;

  constructor(
    readonly game: GameWasmState,
    private scene: THREE.Scene,
    private water: Water
  ) {
    const geometry = new THREE.SphereGeometry(1, 16, 16);
    const material = new THREE.MeshPhongMaterial({
      color: 0xffffff,
      shininess: 80,
      emissiveIntensity: 10,
    });
    this.bulletModel = new THREE.InstancedMesh(geometry, material, 500);
    this.bulletModel.frustumCulled = false;
    this.bulletModel.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.scene.add(this.bulletModel);
    this.explosionManager = new ExplosionManager(scene);
    this.arrowHelper.visible = this.showArrow;
    this.arrowHelper.setLength(10);
    this.scene.add(this.arrowHelper);
    this.outlines = this.boatMesh.clone();

    this.loadModel();
  }

  async loadModel() {
    const loader = new GLTFLoader();
    const obj = await new Promise<THREE.Group<THREE.Object3DEventMap>>(
      (resolve) =>
        loader.load(boat, (_obj) => {
          resolve(_obj.scene);
        })
    );
    const material = new THREE.MeshPhongMaterial({
      color: 0xffffff,
      shininess: 20,
    });

    const mesh = obj.children[0] as THREE.Mesh;

    mesh.geometry.scale(200, 200, 200);
    mesh.geometry.translate(0, -2, 3.5);
    const instancedMesh = new THREE.InstancedMesh(mesh.geometry, material, 500);
    // obj.scale.set(this.scale, this.scale, this.scale);
    // obj.rotation.set(Math.PI / 2, 0, 0);
    instancedMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.boatMesh = instancedMesh;
    this.boatMesh.frustumCulled = false;
    this.outlines = this.boatMesh.clone();
    this.scene.add(this.boatMesh);
    this.scene.add(this.outlines);
  }

  createShip(x: number, y: number) {
    this.game.action_create_ship(x, y);
  }

  getPathTo(x: number, y: number) {
    const pathStr = this.game.find_path(0, 0, x, y);
    if (pathStr) {
      const path: [number, number][] = JSON.parse(pathStr);
      return path;
    }
    return null;
  }

  selectBoat(id: number) {
    this.game.action_selec_ship(id);
  }

  clearSelection() {
    this.game.action_clear_selected();
  }

  getBoatAt(x: number, y: number) {
    for (const ship of this.myShips()) {
      const distance = Math.sqrt(
        (ship.position[0] - x) ** 2 + (ship.position[1] - y) ** 2
      );
      if (distance < SHIP_SIZE) {
        return ship.id;
      }
    }
    return null;
  }

  getMyShip(id: number) {
    for (const ship of this.myShips()) {
      if (ship.id === id) {
        return ship;
      }
    }
    return null;
  }

  *myShips() {
    const myID = this.game.my_id();
    for (const ship of this.ships) {
      if (ship.player_id === myID) {
        yield ship;
      }
    }
  }

  *selectedShips() {
    for (const ship of this.myShips()) {
      if (this.selected.includes(ship.id)) {
        yield ship;
      }
    }
  }

  shoot(x: number, y: number) {
    this.game.action_shoot_at(x, y);
  }

  moveSelected(x: number, y: number) {
    for (const ship of this.selectedShips()) {
      this.game.action_move_ship(ship.id, x, y);
    }
  }

  tick(time: number) {
    if (!this.boatMesh) {
      return;
    }
    const ships: ShipData[] = this.game.get_all_ships();
    const bullets: Bullet[] = this.game.get_all_bullets();
    this.selected = this.game.get_selected_ships();

    this.bulletModel.count = bullets.length;
    this.bulletModel.instanceMatrix.needsUpdate = true;
    if (this.bulletModel.instanceColor) {
      this.bulletModel.instanceColor.needsUpdate = true;
    }
    const matrix = new THREE.Matrix4();
    for (let i = 0; i < bullets.length; i++) {
      matrix.setPosition(
        bullets[i].position[0],
        bullets[i].position[1],
        bullets[i].position[2]
      );
      this.bulletModel.setColorAt(i, this.playerColor(bullets[i].player_id));
      this.bulletModel.setMatrixAt(i, matrix);
    }

    this.boatMesh.instanceMatrix.needsUpdate = true;
    if (this.boatMesh.instanceColor) {
      this.boatMesh.instanceColor.needsUpdate = true;
    }
    this.outlines.instanceMatrix.needsUpdate = true;
    if (this.outlines.instanceColor) {
      this.outlines.instanceColor.needsUpdate = true;
    }

    const myID = this.game.my_id();
    let normalBoats = 0;
    let outlineBoats = 0;
    for (let _i = 0; _i < ships.length; _i++) {
      const ship = ships[_i];
      const isMine = ship.player_id === myID;
      let meshToUse;
      let i;
      if (isMine && this.selected.includes(ship.id)) {
        i = outlineBoats;
        outlineBoats += 1;
        meshToUse = this.outlines;
      } else {
        i = normalBoats;
        normalBoats += 1;
        meshToUse = this.boatMesh;
      }
      this.calcBoatAngle(ship, matrix);
      meshToUse.setMatrixAt(i, matrix);
      const color = this.playerColor(ship.player_id);
      meshToUse.setColorAt(i, color);
    }
    this.boatMesh.count = normalBoats;
    this.outlines.count = outlineBoats;
    this.ships = ships;

    //==== explosions

    const explosions: ExplosionData[] = this.game.get_all_explosions();
    this.explosionManager.tick(time);
    explosions.forEach((explosion) => {
      this.explosionManager.explodeData(
        explosion,
        this.playerColor(explosion.player_id)
      );
    });

    this.selected$.next(this.outlines);
  }

  private playerColor(playerID: number) {
    return playerArray[playerID % playerArray.length];
  }

  private calcBoatAngle(ship: ShipData, matrix: THREE.Matrix4) {
    const [zPos, normal] = this.water.calcElevationAt(
      ship.position[0],
      ship.position[1]
    );
    const xyAngle =
      Math.atan2(ship.orientation[1], ship.orientation[0]) + Math.PI / 2;
    const quaternion = new THREE.Quaternion().setFromUnitVectors(up, normal);
    matrix.makeRotationZ(xyAngle);
    matrix.multiplyMatrices(
      new THREE.Matrix4().makeRotationFromQuaternion(quaternion),
      matrix
    );
    if (this.showArrow) {
      this.arrowHelper.position.set(ship.position[0], ship.position[1], zPos);
      this.arrowHelper.setDirection(normal);
    }
    matrix.setPosition(ship.position[0], ship.position[1], zPos);
  }
}

const P1 = new THREE.Color("#1b69cf");
const P2 = new THREE.Color("#e43131");
const P3 = new THREE.Color("#35d435");
const P4 = new THREE.Color("#d8d840");
const P5 = new THREE.Color("#d643d6");
const P6 = new THREE.Color("#43d8d8");
const playerArray = [P1, P2, P3, P4, P5, P6];
