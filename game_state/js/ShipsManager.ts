import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import boat from "./assets/shipuv.glb?url";
import { ExplosionManager } from "./Particles";
import { Water } from "./Water";
import { RenderOrder } from "./RenderOrder";
import { HPBar } from "./HPBar";
import { Bullet, ExplosionData, PlayerState, ShipData } from "./RustWorldTypes";
import { flagColors, getFlagTexture } from "./PlayerStuff";
import { IslandsManager } from "./IslandsManager";

const SHIP_SIZE = 10;
const MAX_INSTANCES = 2_000;

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
  private ships: ShipData[] = [];
  private colorMap = new Map<number, THREE.Color>();
  private sailsMap = new Map<number, THREE.InstancedMesh>();
  private sailsGeometry: THREE.BufferGeometry | null = null;

  selectionRectangle: THREE.Mesh;
  aimCircle;
  hpBar = new HPBar();
  islandsManager: IslandsManager;

  constructor(
    readonly game: GameWasmState,
    public scene: THREE.Scene,
    private water: Water,
    private camera: THREE.Camera
  ) {
    const geometry = new THREE.SphereGeometry(1, 16, 16);
    const material = new THREE.MeshPhongMaterial({
      color: 0x000000,
      shininess: 80,
    });
    this.islandsManager = new IslandsManager(game, scene);
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
      opacity: 0.3,
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

    this.loadModel();
  }

  private sailMeshOfPlayer(playerID: number) {
    const cachedSails = this.sailsMap.get(playerID);
    if (cachedSails) {
      return cachedSails;
    }
    if (!this.sailsGeometry) {
      return null;
    }
    const flagTexture = getFlagTexture(
      this.game.get_player_flag(BigInt(playerID))
    )?.clone();
    if (!flagTexture) {
      return null;
    }
    flagTexture.flipY = false;
    const sails = new THREE.InstancedMesh(
      this.sailsGeometry,
      new THREE.MeshLambertMaterial({ color: 0xaaaaaa, map: flagTexture }),
      MAX_INSTANCES
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
      MAX_INSTANCES
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

  private updatePlayerColors() {
    const players: PlayerState[] = this.game.get_players();
    players.forEach((player) => {
      const cachedColor = this.colorMap.get(player.id);
      if (cachedColor) {
        return cachedColor;
      }
      const country = this.game.get_player_flag(BigInt(player.id));
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

  tick(time: number) {
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
        bullets[i].position[0],
        bullets[i].position[1],
        bullets[i].position[2]
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

    const cameraPosition = this.camera.position;

    this.resestSailCounts();

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
