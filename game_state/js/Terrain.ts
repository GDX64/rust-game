import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { RenderOrder } from "./RenderOrder";

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
    scene.add(this.minimap.mapMesh);
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
}

class TerrainChunk {
  constructor(
    private gameState: GameWasmState,
    private readonly segments: number,
    public readonly planeMesh: THREE.Mesh
  ) {}

  static new(gameState: GameWasmState, position: THREE.Vector3) {
    const segments = (PLANE_WIDTH / 1000) * SEGMENTS_PER_KM;
    const planeGeometry = new THREE.PlaneGeometry(
      PLANE_WIDTH,
      PLANE_WIDTH,
      segments - 1,
      segments - 1
    );

    // scene.fog = new THREE.Fog(0x999999, 0, 100);

    const planeMaterial = new THREE.MeshLambertMaterial({
      vertexColors: true,
    });
    const colorsBuffer = new Float32Array(segments * segments * 3);
    planeGeometry.setAttribute(
      "color",
      new THREE.BufferAttribute(colorsBuffer, 3)
    );

    const plane = new THREE.Mesh(planeGeometry, planeMaterial);
    plane.position.set(position.x, position.y, position.z);

    const chunk = new TerrainChunk(gameState, segments, plane);
    chunk.updateMesh();
    return chunk;
  }

  updateMesh() {
    const { geometry } = this.planeMesh;
    const arr = geometry.attributes.position.array;
    const colors = geometry.attributes.color.array;
    const sand = new THREE.Color("#beb76f");
    const grass = new THREE.Color("#1e4e1e");
    const rock = new THREE.Color("#382323");
    const oceanBottom = new THREE.Color("#010b13");
    for (let x = 0; x < this.segments; x += 1) {
      for (let y = 0; y < this.segments; y += 1) {
        const i = (y * this.segments + x) * 3;
        const yProportion = y / this.segments;
        let xWorld = (x / this.segments) * PLANE_WIDTH - PLANE_WIDTH / 2;
        let yWorld = (0.5 - yProportion) * PLANE_WIDTH;
        xWorld += this.planeMesh.position.x;
        yWorld += this.planeMesh.position.y;
        let height = this.gameState.get_land_value(xWorld, yWorld);

        arr[i + 2] = height;
        let thisColor;
        if (height < -50) {
          thisColor = oceanBottom;
        } else if (height < 10) {
          thisColor = sand;
        } else if (height < 40) {
          thisColor = grass;
        } else {
          thisColor = rock;
        }
        colors[i] = thisColor.r;
        colors[i + 1] = thisColor.g;
        colors[i + 2] = thisColor.b;
      }
    }
    geometry.attributes.position.needsUpdate = true;
    geometry.attributes.color.needsUpdate = true;
    geometry.computeVertexNormals();
  }
}

class MiniMap {
  mapMesh;
  constructor(game: GameWasmState) {
    const terrain = game.uint_terrain();
    const imgData = new Uint8ClampedArray(terrain.length * 4);
    for (let i = 0; i < terrain.length; i++) {
      const dataIndex = i * 4;
      const terrainValue = terrain[i];
      imgData[dataIndex] = terrainValue * 255;
      imgData[dataIndex + 1] = terrainValue * 255;
      imgData[dataIndex + 2] = terrainValue * 255;
      imgData[dataIndex + 3] = 255;
    }
    console.log("terrain", imgData);
    const dim = Math.sqrt(terrain.length);
    const canvasTexture = new THREE.DataTexture(
      imgData,
      dim,
      dim,
      THREE.RGBAFormat
    );
    canvasTexture.needsUpdate = true;
    const material = new THREE.ShaderMaterial({
      fragmentShader: /*glsl*/ `
      varying vec2 vUv;
      uniform sampler2D canvasTexture;
      void main() {
        vec4 tex = texture2D(canvasTexture, vUv);
        tex.a = 0.8;
        //tex.x = vUv.x;
        //tex.y = vUv.y;
        gl_FragColor = tex;
        //gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }
      `,
      vertexShader: /*glsl*/ `
      varying vec2 vUv;
      uniform float width;
      uniform float height;
      uniform float screenWidth;
      uniform float screenHeight;
      
      void main() {
        vUv = uv;
        vec4 pos = vec4(position, 1.0);
        float boxWidth = width / screenWidth ;
        float boxHeight = height / screenHeight;
        pos.x = (pos.x + 0.5)*boxWidth + 1.0 - boxWidth;
        pos.y = (pos.y - 0.5)*boxHeight - 1.0 + boxHeight;
        pos.z = .5;
        gl_Position = pos;
      }
      `,
      uniforms: {
        canvasTexture: { value: canvasTexture },
        width: { value: 500 },
        height: { value: 500 },
        screenWidth: { value: window.innerWidth },
        screenHeight: { value: window.innerHeight },
      },
      depthTest: false,
      depthWrite: false,
      transparent: true,
    });
    const geometry = new THREE.PlaneGeometry(1, 1);
    const minimap = new THREE.Mesh(geometry, material);
    minimap.renderOrder = RenderOrder.MINIMAP;
    this.mapMesh = minimap;
  }
}
