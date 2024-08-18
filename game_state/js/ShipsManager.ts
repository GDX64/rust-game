import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/boat.glb?url";
import { ExplosionManager } from "./Particles";
import { Water } from "./Water";
import { RenderOrder } from "./RenderOrder";
import { HPBar } from "./HPBar";
import brazil from "./assets/brasil.png";
import {
  Bullet,
  ExplosionData,
  IslandData,
  IslandOwners,
  ShipData,
} from "./RustWorldTypes";

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
  flagSprites = new Map<bigint, THREE.Sprite>();
  private explosionManager: ExplosionManager;
  private bulletModel: THREE.InstancedMesh;
  private ships: ShipData[] = [];
  selectionRectangle: THREE.Mesh;
  aimCircle;
  hpBar = new HPBar();

  constructor(
    readonly game: GameWasmState,
    public scene: THREE.Scene,
    private water: Water
  ) {
    const geometry = new THREE.SphereGeometry(1, 16, 16);
    const material = new THREE.MeshPhongMaterial({
      color: 0xffffff,
      shininess: 80,
      emissiveIntensity: 10,
    });
    this.bulletModel = new THREE.InstancedMesh(geometry, material, 10000);
    this.bulletModel.frustumCulled = false;
    this.bulletModel.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.scene.add(this.bulletModel);
    this.explosionManager = new ExplosionManager(scene);

    const outlineGeometry = new THREE.CircleGeometry(6, 32);
    const outlineMaterial = new THREE.MeshLambertMaterial({
      color: "#ffff00",
      blending: THREE.NormalBlending,
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0.5,
      depthWrite: false,
    });
    this.outlines = new THREE.InstancedMesh(
      outlineGeometry,
      outlineMaterial,
      10000
    );
    this.outlines.frustumCulled = false;
    this.outlines.renderOrder = RenderOrder.OUTLINE;

    const circle = new THREE.CircleGeometry(1, 32);
    const circleMaterial = new THREE.MeshPhongMaterial({
      color: 0xffff00,
      blending: THREE.NormalBlending,
      transparent: true,
      opacity: 0.1,
      depthWrite: false,
    });
    this.aimCircle = new THREE.Mesh(circle, circleMaterial);
    this.aimCircle.position.set(0, 0, 0);
    this.aimCircle.renderOrder = RenderOrder.AIM;
    this.aimCircle.visible = false;

    const selectionRect = new THREE.PlaneGeometry(1, 1);
    selectionRect.translate(0.5, 0.5, 0);
    const selectionRectMaterial = new THREE.MeshBasicMaterial({
      color: 0xffff00,
      blending: THREE.NormalBlending,
      transparent: true,
      opacity: 0.1,
      depthWrite: false,
    });
    this.selectionRectangle = new THREE.Mesh(
      selectionRect,
      selectionRectMaterial
    );
    this.selectionRectangle.renderOrder = RenderOrder.AIM;
    this.selectionRectangle.visible = false;

    this.scene.add(this.selectionRectangle);
    this.scene.add(this.aimCircle);
    this.hpBar.addToScene(scene);

    const { spriteGroup, owners, spriteMap } = this.makeFlags();
    this.flagSprites = spriteMap;
    this.scene.add(spriteGroup);

    this.loadModel();
  }

  makeFlags() {
    const owners: IslandOwners = this.game.island_owners();
    const islandData: IslandData[] = this.game.all_island_data();

    const textureLoader = new THREE.TextureLoader();
    const flagTexture = textureLoader.load(brazil);
    const material = new THREE.SpriteMaterial({
      color: 0xffffff,
      map: flagTexture,
    });

    const sprites = islandData.map((island) => {
      const sprite = new THREE.Sprite(material);
      sprite.scale.set(50, 35, 1);
      sprite.position.set(island.center[0], island.center[1], 150);
      return { sprite, island: island.id };
    });
    const spriteGroup = new THREE.Group();
    spriteGroup.add(...sprites.map(({ sprite }) => sprite));
    const spriteMap = new Map(
      sprites.map(({ island, sprite }) => [island, sprite])
    );
    return { spriteMap, spriteGroup, owners };
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
    const instancedMesh = new THREE.InstancedMesh(
      mesh.geometry,
      material,
      10000
    );
    // obj.scale.set(this.scale, this.scale, this.scale);
    // obj.rotation.set(Math.PI / 2, 0, 0);
    instancedMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.boatMesh = instancedMesh;
    this.boatMesh.frustumCulled = false;
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
    this.game.move_selected_ships(x, y);
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

    this.hpBar.setInstancesCount(ships.length);

    const myID = this.game.my_id();
    let outlineBoats = 0;
    for (let i = 0; i < ships.length; i++) {
      const ship = ships[i];
      const isMine = ship.player_id === myID;
      const meshToUse = this.boatMesh;
      this.calcBoatAngle(ship, matrix);
      meshToUse.setMatrixAt(i, matrix);
      const color = this.playerColor(ship.player_id);
      meshToUse.setColorAt(i, color);
      this.hpBar.updateBar(i, matrix, ship.hp);
      if (isMine && this.selected.includes(ship.id)) {
        this.outlines.setMatrixAt(outlineBoats, matrix);
        outlineBoats++;
      }
    }
    this.boatMesh.count = ships.length;
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
  }

  auto_shoot() {
    this.game.auto_shoot();
  }

  select(fn: (ship: ShipData) => boolean) {
    for (const ship of this.myShips()) {
      if (fn(ship)) {
        this.selectBoat(ship.id);
      }
    }
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
