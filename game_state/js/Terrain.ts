import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { playerColor } from "./PlayerStuff";
import { Linscale } from "./Linscale";

const PLANE_WIDTH = 5_000; //1km
const SEGMENTS_PER_KM = 50;
const minimapPercentage = 0.15;

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

  mapSizeInPixels = Math.floor(window.innerWidth * minimapPercentage);
  constructor(private game: GameWasmState) {
    const mapCanvas = document.createElement("canvas");
    mapCanvas.classList.add("minimap-canvas");
    mapCanvas.width = this.mapSizeInPixels;
    mapCanvas.height = this.mapSizeInPixels;

    const islandsCanvas = new OffscreenCanvas(
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );

    this.islandsCanvas = islandsCanvas;
    this.mapCanvas = mapCanvas;

    this.updateMiniMap();
  }

  updateMiniMap() {
    const terrain = this.game.uint_terrain();
    const imgData = new Uint8ClampedArray(terrain.length * 4);
    for (let i = 0; i < terrain.length; i++) {
      const dataIndex = i * 4;
      const terrainValue = terrain[i];
      let color = 0;
      let alpha = 150;
      if (terrainValue === -1) {
        color = 0;
        alpha = 50;
      } else if (terrainValue === -2) {
        color = 0xffffff;
      } else {
        const threeColor = playerColor(terrainValue);
        color = threeColor.getHex();
      }
      imgData[dataIndex] = color >> 16;
      imgData[dataIndex + 1] = (color >> 8) & 0xff;
      imgData[dataIndex + 2] = color & 0xff;
      imgData[dataIndex + 3] = alpha;
    }

    const dim = Math.sqrt(terrain.length);
    const imgDataArray = new ImageData(imgData, dim, dim);
    createImageBitmap(imgDataArray).then((bitmap) => {
      const ctx = this.islandsCanvas.getContext("2d")!;
      ctx.save();
      ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
      ctx.scale(1, -1);
      ctx.translate(0, -this.mapSizeInPixels);
      ctx.drawImage(bitmap, 0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
      ctx.restore();

      const ctx2 = this.mapCanvas.getContext("2d")!;
      ctx2.drawImage(this.islandsCanvas, 0, 0);
    });
  }

  updateCanvas(camera: THREE.Camera) {
    if (this.game.has_map_changed()) {
      this.updateMiniMap();
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

    ctx.beginPath();
    ctx.moveTo(0, -3);
    ctx.lineTo(10, 0);
    ctx.lineTo(0, 3);
    ctx.fill();
    ctx.restore();
  }
}
