import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/ship.glb?url";

type Ship3D = THREE.Group<THREE.Object3DEventMap>;
type ShipData = {
  player_id: number;
  id: number;
  position: [number, number];
  speed: [number, number];
};

type Ship = {
  data: ShipData;
  model: Ship3D;
};

export class ShipsManager {
  boatModel: Ship3D | null = null;
  ships: Map<string, Ship> = new Map();
  constructor(
    private game: GameWasmState,
    private scale: number,
    private scene: THREE.Scene
  ) {
    const loader = new GLTFLoader();
    loader.load(boat, (_obj) => {
      const obj = _obj.scene;
      obj.scale.set(this.scale, this.scale, this.scale);
      console.log(_obj.animations);
      obj.rotation.set(Math.PI / 2, 0, 0);
      this.boatModel = obj;
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

  async moveShip(x: number, y: number) {
    const first = this.myShips().next().value;
    if (first) {
      this.game.action_move_ship(first.data.id, x, y);
    } else {
      this.createShip(0, 0);
    }
  }

  tick() {}

  update() {
    const ships: ShipData[] = JSON.parse(this.game.get_all_ships());
    ships.forEach((ship) => {
      const key = `${ship.player_id}_${ship.id}`;
      const existing = this.ships.get(key);
      if (existing) {
        existing.model.position.set(ship.position[0], ship.position[1], 0);
        if (ship.speed[0] !== 0 || ship.speed[1] !== 0) {
          const xyAngle =
            Math.atan2(ship.speed[1], ship.speed[0]) + Math.PI / 2;
          existing.model.rotation.set(Math.PI / 2, 0, xyAngle, "ZXY");
        }
      } else if (this.boatModel) {
        const newShip = this.boatModel.clone();
        newShip.position.set(ship.position[0], ship.position[1], 0);
        this.ships.set(key, { model: newShip, data: ship });
        this.scene.add(newShip);
      }
    });
  }
}
