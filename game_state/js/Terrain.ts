import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";
import { RenderOrder } from "./RenderOrder";
import { playerColor } from "./PlayerStuff";

const PLANE_WIDTH = 5_000; //1km
const SEGMENTS_PER_KM = 50;
const minimapPercentage = 0.2;

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
  islandsCanvas;
  mapCanvas;
  canvasTexture;

  mapSizeInPixels = Math.floor(window.innerWidth * minimapPercentage);
  constructor(private game: GameWasmState) {
    const mapCanvas = new OffscreenCanvas(
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );
    const islandsCanvas = new OffscreenCanvas(
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );

    const canvasTexture = new THREE.CanvasTexture(mapCanvas);

    const material = drawImageShader(
      canvasTexture,
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );
    const geometry = new THREE.PlaneGeometry(1, 1);
    const minimap = new THREE.Mesh(geometry, material);
    new THREE.SpriteMaterial({});
    minimap.renderOrder = RenderOrder.MINIMAP;
    minimap.frustumCulled = false;

    this.mapMesh = minimap;
    this.islandsCanvas = islandsCanvas;
    this.mapCanvas = mapCanvas;
    this.canvasTexture = canvasTexture;

    this.updateMiniMap();
  }

  updateMiniMap() {
    const terrain = this.game.uint_terrain();
    const imgData = new Uint8ClampedArray(terrain.length * 4);
    for (let i = 0; i < terrain.length; i++) {
      const dataIndex = i * 4;
      const terrainValue = terrain[i];
      let color = 0;
      if (terrainValue === -1) {
        color = 0;
      } else if (terrainValue === -2) {
        color = 0xffffff;
      } else {
        const threeColor = playerColor(terrainValue);
        color = threeColor.getHex();
      }
      imgData[dataIndex] = color >> 16;
      imgData[dataIndex + 1] = (color >> 8) & 0xff;
      imgData[dataIndex + 2] = color & 0xff;
      imgData[dataIndex + 3] = 255;
    }

    const dim = Math.sqrt(terrain.length);
    const imgDataArray = new ImageData(imgData, dim, dim);
    createImageBitmap(imgDataArray).then((bitmap) => {
      const ctx = this.islandsCanvas.getContext("2d")!;
      ctx.save();
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
      console.log("map changed");
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

    // const drawMatrix = new DOMMatrix();
    // drawMatrix.rotateSelf(rotationOnXY * (180 / Math.PI));
    // drawMatrix.e = xOnCanvas;
    // drawMatrix.f = yOnCanvas;

    ctx.translate(xOnCanvas, yOnCanvas);
    ctx.rotate(rotationOnXY);

    ctx.beginPath();
    ctx.moveTo(0, -3);
    ctx.lineTo(10, 0);
    ctx.lineTo(0, 3);
    ctx.fill();
    ctx.restore();
    this.canvasTexture.needsUpdate = true;
  }
}

function drawImageShader(
  texture: THREE.Texture,
  width: number,
  height: number
) {
  const material = new THREE.ShaderMaterial({
    fragmentShader: /*glsl*/ `
      varying vec2 vUv;
      uniform sampler2D canvasTexture;
      void main() {
        vec4 tex = texture2D(canvasTexture, vUv);
        tex.a = 0.5;
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
        float boxWidth = 2.0*width / (screenWidth) ;
        float boxHeight = 2.0*height / (screenHeight);
        pos.x = (pos.x + 0.5)*boxWidth + 1.0 - boxWidth;
        pos.y = (pos.y - 0.5)*boxHeight - 1.0 + boxHeight;
        pos.z = .5;
        gl_Position = pos;
      }
      `,
    uniforms: {
      canvasTexture: { value: texture },
      width: { value: width },
      height: { value: height },
      screenWidth: { value: window.innerWidth },
      screenHeight: { value: window.innerHeight },
    },
    depthTest: false,
    depthWrite: false,
    transparent: true,
  });
  return material;
}
