import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/boat.glb?url";
import { ExplosionData, ExplosionManager } from "./Particles";

type ShipData = {
  player_id: number;
  id: number;
  position: [number, number];
  speed: [number, number];
  acceleration: [number, number];
  orientation: [number, number];
};

// type Diff<K, T> =
//   | {
//       Update: [K, T];
//     }
//   | {
//       Remove: K;
//     }
//   | {
//       Add: [K, T];
//     };

// type StateDiff = {
//   bullets: Diff<[number, number], Bullet>[];
// };

type Bullet = {
  position: [number, number, number];
  speed: [number, number, number];
  id: number;
  player_id: number;
};

export class ShipsManager {
  boatMesh: THREE.InstancedMesh | null = null;
  explosionManager: ExplosionManager;
  bulletModel: THREE.InstancedMesh;
  ships: ShipData[] = [];
  constructor(
    private game: GameWasmState,
    private scale: number,
    private scene: THREE.Scene,
    private camera: THREE.Camera
  ) {
    const geometry = new THREE.SphereGeometry(1, 16, 16);
    const material = new THREE.MeshPhongMaterial({
      color: 0xffffff,
      shininess: 80,
      emissiveIntensity: 10,
    });
    this.bulletModel = new THREE.InstancedMesh(geometry, material, 500);
    this.bulletModel.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.scene.add(this.bulletModel);
    this.explosionManager = new ExplosionManager(scene);

    document.addEventListener("keydown", (event) => {
      if (event.key === "s") {
      }
    });
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
    mesh.geometry.translate(0, 0, 4);
    const instancedMesh = new THREE.InstancedMesh(mesh.geometry, material, 500);
    // obj.scale.set(this.scale, this.scale, this.scale);
    // obj.rotation.set(Math.PI / 2, 0, 0);
    instancedMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.boatMesh = instancedMesh;
    this.scene.add(instancedMesh);
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

  *myShips() {
    const myID = this.game.my_id();
    for (const ship of this.ships) {
      if (ship.player_id === myID) {
        yield ship;
      }
    }
  }

  shoot(x: number, y: number) {
    this.game.shoot_with_all(
      x,
      y,
      this.camera.position.x,
      this.camera.position.y
    );
  }

  moveShip(x: number, y: number) {
    const first = this.myShips().next().value;
    if (first) {
      this.game.action_move_ship(first.id, x, y);
    } else {
      this.createShip(0, 0);
    }
  }

  tick() {
    if (!this.boatMesh) {
      return;
    }
    const ships: ShipData[] = this.game.get_all_ships();
    const bullets: Bullet[] = this.game.get_all_bullets();
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

    this.boatMesh.count = ships.length;
    this.boatMesh.instanceMatrix.needsUpdate = true;
    if (this.boatMesh.instanceColor) {
      this.boatMesh.instanceColor.needsUpdate = true;
    }
    for (let i = 0; i < ships.length; i++) {
      const ship = ships[i];
      calcBoatAngle(ship, matrix);
      matrix.setPosition(ship.position[0], ship.position[1], 0);
      this.boatMesh.setMatrixAt(i, matrix);
      this.boatMesh.setColorAt(i, this.playerColor(ship.player_id));
    }
    this.ships = ships;

    //==== explosions

    const explosions: ExplosionData[] = this.game.get_all_explosions();
    this.explosionManager.tick(0.016);
    explosions.forEach((explosion) => {
      this.explosionManager.explodeData(
        explosion,
        this.playerColor(explosion.player_id)
      );
    });
  }

  private playerColor(playerID: number) {
    return playerArray[playerID % playerArray.length];
  }
}

const P1 = new THREE.Color("#e43131");
const P2 = new THREE.Color("#1b69cf");
const P3 = new THREE.Color("#35d435");
const P4 = new THREE.Color("#d8d840");
const P5 = new THREE.Color("#d643d6");
const P6 = new THREE.Color("#43d8d8");
const playerArray = [P1, P2, P3, P4, P5, P6];

function calcBoatAngle(ship: ShipData, matrix: THREE.Matrix4) {
  const xyAngle =
    Math.atan2(ship.orientation[1], ship.orientation[0]) + Math.PI / 2;
  matrix.makeRotationZ(xyAngle);
}
