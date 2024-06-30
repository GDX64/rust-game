import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/ship.glb?url";

type Ship3D = THREE.Group<THREE.Object3DEventMap>;
type ShipData = {
  player_id: number;
  id: number;
  position: [number, number];
};

type Ship = {
  data: ShipData;
  model: Ship3D;
};

export class ShipsManager {
  boatModel: Ship3D | null = null;
  ships: Map<number, Ship> = new Map();
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

  async moveShip(x: number, y: number) {
    const first: Ship = this.ships.values().next().value;
    if (!first) {
      this.createShip(0, 0);
    }
    this.game.action_move_ship(first.data.id, x, y);
  }

  tick() {}

  update() {
    const ships: ShipData[] = JSON.parse(this.game.get_all_ships());
    ships.forEach((ship) => {
      const existing = this.ships.get(ship.id);
      if (existing) {
        existing.model.position.set(ship.position[0], ship.position[1], 0);
      } else if (this.boatModel) {
        const newShip = this.boatModel.clone();
        newShip.position.set(ship.position[0], ship.position[1], 0);
        this.ships.set(ship.id, { model: newShip, data: ship });
        this.scene.add(newShip);
      }
    });
  }
}
