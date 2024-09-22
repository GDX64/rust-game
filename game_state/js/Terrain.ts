import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { Linscale } from "./Linscale";
import { MiniMap } from "./MiniMap";

const PLANE_WIDTH = 5_000; //1km
const SEGMENTS_PER_KM = 50;

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
