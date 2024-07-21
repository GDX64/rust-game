import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/ship.glb?url";
import { ExplosionData, ExplosionManager } from "./Particles";

type Ship3D = THREE.Group<THREE.Object3DEventMap>;
type ShipData = {
  player_id: number;
  id: number;
  position: [number, number];
  speed: [number, number];
  acceleration: [number, number];
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
};

type Ship = {
  data: ShipData;
  model: Ship3D;
  visitedThisFrame: boolean;
};

export class ShipsManager {
  boatModel: Ship3D | null = null;
  explosionManager: ExplosionManager;
  bulletModel: THREE.InstancedMesh;
  ships: Map<string, Ship> = new Map();
  constructor(
    private game: GameWasmState,
    private scale: number,
    private scene: THREE.Scene,
    private camera: THREE.Camera
  ) {
    const loader = new GLTFLoader();
    loader.load(boat, (_obj) => {
      const obj = _obj.scene;
      console.log(obj.children[0]);
      obj.scale.set(this.scale, this.scale, this.scale);
      console.log(_obj.animations);
      obj.rotation.set(Math.PI / 2, 0, 0);
      this.boatModel = obj;
    });

    const geometry = new THREE.SphereGeometry(1, 16, 16);
    const material = new THREE.MeshLambertMaterial({
      color: 0xffff00,
      side: THREE.DoubleSide,
    });
    // const referenceSphere = new THREE.Mesh(
    //   new THREE.SphereGeometry(10, 16, 16),
    //   material
    // );
    // this.scene.add(referenceSphere);
    this.bulletModel = new THREE.InstancedMesh(geometry, material, 100);
    this.scene.add(this.bulletModel);
    this.explosionManager = new ExplosionManager(scene);

    document.addEventListener("keydown", (event) => {
      if (event.key === "s") {
      }
    });
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
    for (const ship of this.ships.values()) {
      if (ship.data.player_id === myID) {
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
      this.game.action_move_ship(first.data.id, x, y);
    } else {
      this.createShip(0, 0);
    }
  }

  update() {
    const ships: ShipData[] = this.game.get_all_ships();
    console.log(
      ships
        .filter((ship) => ship.player_id === this.game.my_id())
        .map((ship) => ship.acceleration)[0]
    );
    const bullets: Bullet[] = this.game.get_all_bullets();
    console.log(bullets);
    this.bulletModel.count = bullets.length;
    const matrix = new THREE.Matrix4();
    for (let i = 0; i < bullets.length; i++) {
      matrix.setPosition(
        bullets[i].position[0],
        bullets[i].position[1],
        bullets[i].position[2]
      );
      this.bulletModel.setMatrixAt(i, matrix);
    }
    this.bulletModel.instanceMatrix.needsUpdate = true;

    ships.forEach((ship) => {
      const key = `${ship.player_id}_${ship.id}`;
      const existing = this.ships.get(key);
      if (existing) {
        existing.model.position.set(ship.position[0], ship.position[1], 0);
        existing.visitedThisFrame = true;
        if (ship.speed[0] !== 0 || ship.speed[1] !== 0) {
          const xyAngle =
            Math.atan2(ship.speed[1], ship.speed[0]) + Math.PI / 2;
          existing.model.rotation.set(Math.PI / 2, 0, xyAngle, "ZXY");
        }
      } else if (this.boatModel) {
        const newShip = this.boatModel.clone();
        newShip.position.set(ship.position[0], ship.position[1], 0);
        this.ships.set(key, {
          model: newShip,
          data: ship,
          visitedThisFrame: true,
        });
        this.scene.add(newShip);
      }
    });
    this.ships.forEach((ship, key) => {
      if (!ship.visitedThisFrame) {
        this.scene.remove(ship.model);
        this.ships.delete(key);
      }
    });
    this.ships.forEach((ship) => {
      ship.visitedThisFrame = false;
    });

    //==== explosions

    const explosions: ExplosionData[] = this.game.get_all_explosions();
    this.explosionManager.tick(0.016);
    explosions.forEach((explosion) => {
      this.explosionManager.explodeData(explosion);
    });
  }
}
