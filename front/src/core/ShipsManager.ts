import { GameWasmState } from "rust";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "../assets/shipuv.glb?url";
import { ExplosionManager } from "./Particles";
import { Water } from "./Water";
import { RenderOrder } from "./RenderOrder";
import { HPBar } from "./HPBar";
import {
  Bullet,
  CenterResults,
  ExplosionData,
  PlayerState,
  ShipData,
} from "./RustWorldTypes";
import { flagColors, getFlagTexture } from "./PlayerStuff";
import { IslandsManager } from "./IslandsManager";
import { CameraControl } from "./CameraControl";

const SHIP_SIZE = 10;
const MAX_SHIPS = 120;
const MAX_PLAYERS = 10;
const ARMY_FLAG_HEIGHT = 50;
const FRAMES_UNTIL_CHECK_DEAD_PLAYERS = 3_000;

const up = new THREE.Vector3(0, 0, 1);
const defaultColor = new THREE.Color(0x999999);

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
  private armyFlags = new Map<number, THREE.Sprite[]>();
  private ships: ShipData[] = [];
  private colorMap = new Map<number, THREE.Color>();
  private sailsMap = new Map<number, THREE.InstancedMesh>();
  private sailsGeometry: THREE.BufferGeometry | null = null;
  private armyFlagsGroup = new THREE.Group();

  aimCircle;
  hpBar = new HPBar();
  islandsManager: IslandsManager;

  constructor(
    readonly game: GameWasmState,
    public scene: THREE.Scene,
    private water: Water,
    private cameraControl: CameraControl
  ) {
    const geometry = new THREE.SphereGeometry(1, 16, 16);
    const material = new THREE.MeshPhongMaterial({
      color: 0x000000,
      shininess: 80,
    });
    this.islandsManager = new IslandsManager(game, scene);
    this.bulletModel = new THREE.InstancedMesh(
      geometry,
      material,
      MAX_PLAYERS * MAX_SHIPS
    );
    this.bulletModel.frustumCulled = false;
    this.bulletModel.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.scene.add(this.bulletModel);
    this.explosionManager = new ExplosionManager(
      scene,
      this.cameraControl.listener
    );

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
      MAX_SHIPS
    );
    this.outlines.frustumCulled = false;
    this.outlines.renderOrder = RenderOrder.OUTLINE;

    const circle = new THREE.CircleGeometry(1, 32);
    const circleMaterial = new THREE.MeshPhongMaterial({
      color: 0xffff00,
      blending: THREE.NormalBlending,
      transparent: true,
      opacity: 0.3,
      depthWrite: false,
    });
    this.aimCircle = new THREE.Mesh(circle, circleMaterial);
    this.aimCircle.position.set(0, 0, 0);
    this.aimCircle.renderOrder = RenderOrder.AIM;
    this.aimCircle.visible = false;

    this.scene.add(this.aimCircle);
    this.hpBar.addToScene(scene);
    this.scene.add(this.armyFlagsGroup);

    this.loadModel();
  }

  get camera() {
    return this.cameraControl.camera;
  }

  private sailMeshOfPlayer(playerID: number) {
    const cachedSails = this.sailsMap.get(playerID);
    if (cachedSails) {
      return cachedSails;
    }
    if (!this.sailsGeometry) {
      return null;
    }
    const country = this.game.get_player_flag(BigInt(playerID));
    if (!country) {
      return null;
    }
    const flagTexture = getFlagTexture(country)?.clone();
    if (!flagTexture) {
      return null;
    }
    flagTexture.flipY = false;
    const sails = new THREE.InstancedMesh(
      this.sailsGeometry,
      new THREE.MeshLambertMaterial({ color: 0xaaaaaa, map: flagTexture }),
      MAX_SHIPS
    );
    sails.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    sails.frustumCulled = false;
    this.sailsMap.set(playerID, sails);
    this.scene.add(sails);
    return sails;
  }

  async loadModel() {
    const loader = new GLTFLoader();
    const obj = await new Promise<THREE.Group<THREE.Object3DEventMap>>(
      (resolve) =>
        loader.load(boat, (_obj) => {
          resolve(_obj.scene);
        })
    );

    const material = new THREE.MeshLambertMaterial({
      color: "#9e9e9e",
    });

    const objChildren = obj.children[0] as THREE.Object3D;
    const [sails, hull] = objChildren.children as THREE.Mesh[];
    sails.geometry.applyMatrix4(sails.matrix).rotateX(Math.PI / 2);
    hull.geometry.applyMatrix4(hull.matrix).rotateX(Math.PI / 2);
    const scaleFactor = 2.2;
    sails.geometry.scale(scaleFactor, scaleFactor, scaleFactor);
    hull.geometry.scale(scaleFactor, scaleFactor, scaleFactor);
    const instancedHulls = new THREE.InstancedMesh(
      hull.geometry,
      material,
      MAX_SHIPS * MAX_PLAYERS
    );

    instancedHulls.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.boatMesh = instancedHulls;
    this.boatMesh.frustumCulled = false;
    this.scene.add(this.boatMesh);
    this.scene.add(this.outlines);
    this.sailsGeometry = sails.geometry;
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
        (ship.position.x - x) ** 2 + (ship.position.y - y) ** 2
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

  private updatePlayerColors() {
    const players: PlayerState[] = this.game.get_players();
    players.forEach((player) => {
      const cachedColor = this.colorMap.get(player.id);
      if (cachedColor) {
        return cachedColor;
      }
      const country = this.game.get_player_flag(BigInt(player.id));

      if (!country) return null;

      const vibrant = flagColors(country);
      if (vibrant) {
        const color = new THREE.Color(vibrant);
        this.colorMap.set(player.id, color);
        return color;
      }
    });
  }

  playerColor(playerID: number) {
    const color = this.colorMap.get(playerID);
    return color ?? defaultColor;
  }

  private resestSailCounts() {
    for (const sails of this.sailsMap.values()) {
      sails.count = 0;
      sails.instanceMatrix.needsUpdate = true;
    }
  }

  private freeDeadPlayerResources() {
    const alivePlayers: Map<number, PlayerState> = this.game.get_all_players();
    this.armyFlags.forEach((flags, playerID) => {
      if (!alivePlayers.has(playerID)) {
        console.log("removing flags for player", playerID);
        flags.forEach((flag) => {
          flag.removeFromParent();
          flag.material.dispose();
        });
        this.armyFlags.delete(playerID);
      }
    });
    this.sailsMap.forEach((sails, playerID) => {
      if (!alivePlayers.has(playerID)) {
        console.log("removing sails for player", playerID);
        sails.removeFromParent();
        if (sails.material instanceof THREE.Material) {
          sails.material.dispose();
        }
        this.sailsMap.delete(playerID);
      }
    });
  }

  private getArmyFlag(player: number, i: number) {
    let flags = this.armyFlags.get(player);
    if (!flags) {
      const country = this.game.get_player_flag(BigInt(player));
      if (!country) return null;

      const flatTexture = getFlagTexture(country);
      if (!flatTexture) {
        return null;
      }
      const material = new THREE.SpriteMaterial({ map: flatTexture });
      const sprite = new THREE.Sprite(material);
      this.armyFlagsGroup.add(sprite);
      const hPerW = 0.67;
      const width = 20;
      sprite.scale.set(width, width * hPerW, 1);
      flags = [sprite];
      this.armyFlags.set(player, flags);
    }
    const thisFlag = flags[i];
    if (!thisFlag) {
      const clone = flags[0].clone();
      flags.push(clone);
      this.armyFlagsGroup.add(clone);
    }
    return flags[i];
  }

  tick(time: number, frames: number) {
    if (!this.boatMesh) {
      return;
    }
    this.updatePlayerColors();
    this.islandsManager.tick();
    const cameraX = this.camera.position.x;
    const cameraY = this.camera.position.y;
    const ships: ShipData[] = this.game.get_all_ships(cameraX, cameraY);
    const bullets: Bullet[] = this.game.get_all_bullets(cameraX, cameraY);
    this.selected = this.game.get_selected_ships();

    this.bulletModel.count = bullets.length;
    this.bulletModel.instanceMatrix.needsUpdate = true;
    if (this.bulletModel.instanceColor) {
      this.bulletModel.instanceColor.needsUpdate = true;
    }
    const matrix = new THREE.Matrix4();
    for (let i = 0; i < bullets.length; i++) {
      matrix.setPosition(
        bullets[i].position.x,
        bullets[i].position.y,
        bullets[i].position.z
      );
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
    let outlineBoats = 0;
    let boatsDrawn = 0;

    this.resestSailCounts();

    const playersDrawn = new Set<number>();

    for (let i = 0; i < ships.length; i++) {
      const ship = ships[i];
      const sail = this.sailMeshOfPlayer(ship.player_id);
      if (!sail) {
        continue;
      }

      const drawIndex = boatsDrawn;

      this.calcBoatAngle(ship, matrix);
      this.boatMesh.setMatrixAt(drawIndex, matrix);
      sail.setMatrixAt(sail.count, matrix);
      this.boatMesh.setColorAt(drawIndex, this.playerColor(ship.player_id));
      sail.count += 1;

      this.hpBar.updateBar(drawIndex, matrix, ship.hp);
      const isMine = ship.player_id === myID;
      if (isMine && this.selected.includes(ship.id)) {
        this.outlines.setMatrixAt(outlineBoats, matrix);
        outlineBoats++;
      }
      boatsDrawn++;

      playersDrawn.add(ship.player_id);
    }
    this.boatMesh.count = boatsDrawn;
    this.outlines.count = outlineBoats;
    this.ships = ships;
    this.hpBar.setInstancesCount(boatsDrawn);

    //==== explosions

    const explosions: ExplosionData[] = this.game.get_all_explosions(
      cameraX,
      cameraY
    );
    this.explosionManager.tick(time);
    explosions.forEach((explosion) => {
      this.explosionManager.explodeData(
        explosion,
        this.playerColor(explosion.player_id)
      );
    });

    // army flags

    //mark all as invisible
    this.armyFlagsGroup.children.forEach((sprite) => {
      sprite.visible = false;
    });

    playersDrawn.forEach((player) => {
      const results: CenterResults[] =
        this.game.get_all_center_of_player_around(player, cameraX, cameraY);
      results.forEach(({ center }, i) => {
        const flag = this.getArmyFlag(player, i);
        if (!flag) {
          return;
        }
        flag.visible = true;
        flag.position.set(center[0], center[1], ARMY_FLAG_HEIGHT);
      });
    });
    if (frames % FRAMES_UNTIL_CHECK_DEAD_PLAYERS === 0) {
      this.freeDeadPlayerResources();
    }
  }

  destroy() {
    this.hpBar.destroy();
    this.explosionManager.destroy();
    this.islandsManager.destroy();
    this.boatMesh.dispose();
    this.bulletModel.dispose();
    this.outlines.dispose();
    this.sailsMap.forEach((sails) => {
      sails.dispose();
    });
    this.armyFlags.forEach((flags) => {
      flags.forEach((flag) => {
        flag.material.dispose();
      });
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

  private calcBoatAngle(ship: ShipData, matrix: THREE.Matrix4) {
    const [zPos, normal] = this.water.calcElevationAt(
      ship.position.x,
      ship.position.y
    );
    const xyAngle =
      Math.atan2(ship.orientation.y, ship.orientation.x) + Math.PI / 2;
    const quaternion = new THREE.Quaternion().setFromUnitVectors(up, normal);
    matrix.makeRotationZ(xyAngle);
    matrix.multiplyMatrices(
      new THREE.Matrix4().makeRotationFromQuaternion(quaternion),
      matrix
    );
    matrix.setPosition(ship.position.x, ship.position.y, zPos);
  }
}
