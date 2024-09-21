import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { Linscale } from "./Linscale";
import { Subject } from "rxjs";
import { IslandData, IslandOwners } from "./RustWorldTypes";
import { getFlagImage } from "./PlayerStuff";

const PLANE_WIDTH = 5_000; //1km
const SEGMENTS_PER_KM = 50;
const minimapPercentage = 1;

export class Terrain {
  minimap;
  terrainGroup = new THREE.Group();
  constructor(
    private gameState: GameWasmState,
    private chunks: TerrainChunk[]
  ) {
    this.minimap = new MiniMap(gameState);
    this.terrainGroup.add(...chunks.map((c) => c.planeMesh));
  }

  addToScene(scene: THREE.Scene) {
    scene.add(this.terrainGroup);
  }

  static new(gameState: GameWasmState) {
    const chunks: TerrainChunk[] = [];
    const mapSize = gameState.map_size();
    const chunksDimension = Math.ceil(mapSize / PLANE_WIDTH);
    for (let i = 0; i < chunksDimension; i++) {
      for (let j = 0; j < chunksDimension; j++) {
        const position = new THREE.Vector3(i * PLANE_WIDTH, j * PLANE_WIDTH, 0);
        chunks.push(TerrainChunk.new(gameState, position));
      }
    }
    return new Terrain(gameState, chunks);
  }

  tick(camera: THREE.Camera) {
    this.minimap.updateCanvas(camera);
  }
}

class TerrainChunk {
  constructor(
    private gameState: GameWasmState,
    private readonly segments: number,
    public readonly planeMesh: THREE.Mesh
  ) {}

  static terrainPalletteTexture(gameState: GameWasmState) {
    const width = 1024;
    const [min, max] = gameState.min_max_height();
    const scale = Linscale.fromPoints(min, 0, max, 1);
    const offCanvas = new OffscreenCanvas(width, 1);
    const ctx = offCanvas.getContext("2d")!;
    const grad = ctx.createLinearGradient(0, 0, width, 0);
    const sand = "#f4e434";
    grad.addColorStop(0, "#010b13");
    grad.addColorStop(scale.scale(-20), "#010b13");
    grad.addColorStop(scale.scale(5), sand);
    grad.addColorStop(scale.scale(20), "#20bc20");
    // grad.addColorStop(scale.scale(30), "#157c15");
    grad.addColorStop(scale.scale(60), "#411313");
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, width, 1);
    return new THREE.CanvasTexture(offCanvas);
  }

  static new(gameState: GameWasmState, position: THREE.Vector3) {
    const segments = (PLANE_WIDTH / 1000) * SEGMENTS_PER_KM;
    const planeGeometry = new THREE.PlaneGeometry(
      PLANE_WIDTH,
      PLANE_WIDTH,
      segments - 1,
      segments - 1
    );

    const texture = TerrainChunk.terrainPalletteTexture(gameState);

    const planeMaterial = new THREE.MeshLambertMaterial({
      // wireframe: true,
      color: 0x666666,
      map: texture,
    });

    // const colorsBuffer = new Float32Array(segments * segments * 3);
    // planeGeometry.setAttribute(
    //   "color",
    //   new THREE.BufferAttribute(colorsBuffer, 3)
    // );

    const plane = new THREE.Mesh(planeGeometry, planeMaterial);
    plane.position.set(position.x, position.y, position.z);

    const chunk = new TerrainChunk(gameState, segments, plane);
    chunk.updateMesh();
    return chunk;
  }

  updateMesh() {
    const { geometry } = this.planeMesh;
    const posArr = geometry.attributes.position.array;
    const uvArr = geometry.attributes.uv.array;
    const [min, max] = this.gameState.min_max_height();
    const heightScale = Linscale.fromPoints(min, 0, max, 1);
    for (let x = 0; x < this.segments; x += 1) {
      for (let y = 0; y < this.segments; y += 1) {
        const i = (y * this.segments + x) * 3;
        const uvIndex = (y * this.segments + x) * 2;
        let xWorld = posArr[i];
        let yWorld = posArr[i + 1];
        xWorld += this.planeMesh.position.x;
        yWorld += this.planeMesh.position.y;
        let height = this.gameState.get_land_value(xWorld, yWorld);

        posArr[i + 2] = height;
        const thisUV = heightScale.scale(height);
        uvArr[uvIndex] = thisUV;
        uvArr[uvIndex + 1] = 0;
      }
    }
    geometry.attributes.position.needsUpdate = true;
    geometry.attributes.uv.needsUpdate = true;
    geometry.computeVertexNormals();
  }
}

class MiniMap {
  islandsCanvas;
  mapCanvas;
  mapClick$ = new Subject<{ x: number; y: number }>();
  needUpdate = false;

  mapSizeInPixels;
  constructor(private game: GameWasmState) {
    const mapCanvas = document.createElement("canvas");
    mapCanvas.classList.add("minimap-canvas");

    this.mapSizeInPixels =
      Math.min(window.innerWidth, window.innerHeight) *
      minimapPercentage *
      devicePixelRatio;

    mapCanvas.width = this.mapSizeInPixels;
    mapCanvas.height = this.mapSizeInPixels;

    mapCanvas.onclick = (event: MouseEvent) => {
      const { height, width } = mapCanvas.getBoundingClientRect();
      const miniMapToWorldX = Linscale.fromPoints(
        0,
        -PLANE_WIDTH / 2,
        width,
        PLANE_WIDTH / 2
      );
      const miniMapToWorldY = Linscale.fromPoints(
        0,
        PLANE_WIDTH / 2,
        height,
        -PLANE_WIDTH / 2
      );
      event.stopPropagation();
      const x = event.offsetX;
      const y = event.offsetY;
      const xWorld = miniMapToWorldX.scale(x);
      const yWorld = miniMapToWorldY.scale(y);
      this.mapClick$.next({ x: xWorld, y: yWorld });
    };

    const islandsCanvas = new OffscreenCanvas(
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );

    this.islandsCanvas = islandsCanvas;
    this.mapCanvas = mapCanvas;
  }

  private updateIslands() {
    const islandData: IslandData[] = this.game.all_island_data();
    const ctx = this.islandsCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);

    const mapSize = this.game.map_size();
    const scaleX = Linscale.fromPoints(
      -mapSize / 2,
      0,
      mapSize / 2,
      this.mapSizeInPixels
    );
    const scaleY = Linscale.fromPoints(
      mapSize / 2,
      0,
      -mapSize / 2,
      this.mapSizeInPixels
    );

    const owners: IslandOwners = this.game.island_owners();
    const errorMargin = scaleX.inverseScale().alpha();
    islandData.forEach((island) => {
      ctx.save();
      const path: [number, number][] = this.game.get_island_path(
        BigInt(island.id),
        errorMargin
      );
      const owner = owners.get(island.id)?.owner;
      ctx.beginPath();
      const x = scaleX.scale(path[0][0]);
      const y = scaleY.scale(path[0][1]);
      ctx.moveTo(x, y);
      let minX = x;
      let minY = y;
      let maxX = x;
      let maxY = y;
      path.slice(1).forEach(([x, y]) => {
        const scaledX = scaleX.scale(x);
        const scaledY = scaleY.scale(y);
        ctx.lineTo(scaledX, scaledY);
        minX = Math.min(minX, scaledX);
        minY = Math.min(minY, scaledY);
        maxX = Math.max(maxX, scaledX);
        maxY = Math.max(maxY, scaledY);
      });

      const islandWidth = maxX - minX;
      const islandHeight = maxY - minY;

      ctx.closePath();
      ctx.lineWidth = 2;
      ctx.strokeStyle = "#000000";
      ctx.stroke();

      if (owner != null) {
        const country = this.game.get_player_flag(BigInt(owner));
        const img = getFlagImage(country);
        if (img.width) {
          ctx.clip();
          const drawHeight = Math.max(islandWidth, islandHeight);
          const aspectRatio = img.width / img.height;
          ctx.drawImage(
            img,
            minX,
            minY,
            Math.floor(drawHeight * aspectRatio),
            drawHeight
          );
        } else {
          img.onload = () => {
            this.needUpdate = true;
          };
        }
      } else {
        ctx.fillStyle = "#616161";
        ctx.fill();
      }
      ctx.restore();
    });
  }

  updateCanvas(camera: THREE.Camera) {
    if (this.game.has_map_changed() || this.needUpdate) {
      this.updateIslands();
      this.needUpdate = false;
    }

    const ctx = this.mapCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
    ctx.drawImage(this.islandsCanvas, 0, 0);

    const cameraPosition = camera.position;
    const cameraDirection = camera.getWorldDirection(new THREE.Vector3());

    //draw triangle for camera
    ctx.fillStyle = "#ffff00";
    const rotationOnXY = Math.atan2(cameraDirection.y, cameraDirection.x);

    ctx.save();

    ctx.scale(1, -1);
    ctx.translate(this.mapSizeInPixels / 2, -this.mapSizeInPixels / 2);

    const xOnCanvas = (cameraPosition.x / PLANE_WIDTH) * this.mapSizeInPixels;
    const yOnCanvas = (cameraPosition.y / PLANE_WIDTH) * this.mapSizeInPixels;

    ctx.translate(xOnCanvas, yOnCanvas);
    ctx.rotate(rotationOnXY);

    const ARROW_SIZE = 9;

    const arrowSize = ARROW_SIZE * devicePixelRatio;
    ctx.beginPath();
    ctx.moveTo(0, -arrowSize / 3);
    ctx.lineTo(arrowSize, 0);
    ctx.lineTo(0, arrowSize / 3);
    ctx.fill();
    ctx.restore();
  }
}
