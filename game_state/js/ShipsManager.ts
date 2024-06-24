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

export class ShipsManager {
  boatModel: Ship3D | null = null;
  ships: Map<number, Ship3D> = new Map();
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

  update() {
    const ships: ShipData[] = JSON.parse(this.game.get_all_ships());
    ships.forEach((ship) => {
      const existing = this.ships.get(ship.id);
      if (existing) {
        existing.position.set(ship.position[0], ship.position[1], 0);
      } else if (this.boatModel) {
        const newShip = this.boatModel.clone();
        newShip.position.set(ship.position[0], ship.position[1], 0);
        newShip.userData.id = ship.id;
        this.ships.set(ship.id, newShip);
        this.scene.add(newShip);
      }
    });
  }
}
