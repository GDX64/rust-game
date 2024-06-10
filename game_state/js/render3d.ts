import * as THREE from "three";
import WebgpuRenderer from "three/addons/renderers/webgpu/WebGPURenderer.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { WorldGen } from "../pkg/game_state";

export class Render3D {
  async init() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );

    const PLANE_WIDTH = 1000;
    const PLANE_SEGMENTS = 1000;
    const planeGeometry = new THREE.PlaneGeometry(
      PLANE_WIDTH,
      PLANE_WIDTH,
      PLANE_SEGMENTS - 1,
      PLANE_SEGMENTS - 1
    );

    const arr = planeGeometry.attributes.position.array;

    const world = WorldGen.new(1);
    const textureMap = new Uint8ClampedArray(
      PLANE_SEGMENTS * PLANE_SEGMENTS * 4
    );
    for (let x = 0; x < PLANE_SEGMENTS; x += 1) {
      for (let y = 0; y < PLANE_SEGMENTS; y += 1) {
        const i = (y * PLANE_SEGMENTS + x) * 3;
        let height = world.get_land_value(x / 100, y / 100) * 40 - 5;
        height = Math.max(height, 0);
        arr[i + 2] = height;

        const textureIndex = (y * PLANE_SEGMENTS + x) * 4;
        if (height > 0) {
          if (height > 15) {
            textureMap[textureIndex] = 170;
            textureMap[textureIndex + 1] = 170;
            textureMap[textureIndex + 2] = 170;
            textureMap[textureIndex + 3] = 255;
          } else {
            textureMap[textureIndex] = 0;
            textureMap[textureIndex + 1] = 255;
            textureMap[textureIndex + 2] = 0;
            textureMap[textureIndex + 3] = 255;
          }
        } else {
          textureMap[textureIndex] = 0;
          textureMap[textureIndex + 1] = 0;
          textureMap[textureIndex + 2] = 255;
          textureMap[textureIndex + 3] = 255;
        }
      }
    }
    planeGeometry.computeVertexNormals();

    //downsample
    const canvas = document.createElement("canvas");
    canvas.width = 100;
    canvas.height = 100;
    const ctx = canvas.getContext("2d")!;
    const img = new ImageData(textureMap, PLANE_SEGMENTS, PLANE_SEGMENTS);
    const imageBitmap = await createImageBitmap(img);
    ctx.drawImage(
      imageBitmap,
      0,
      0,
      PLANE_SEGMENTS,
      PLANE_SEGMENTS,
      0,
      0,
      100,
      100
    );

    const planeTexture = new THREE.CanvasTexture(canvas);
    planeTexture.wrapS = THREE.ClampToEdgeWrapping;
    planeTexture.wrapT = THREE.ClampToEdgeWrapping;
    planeTexture.needsUpdate = true;

    const planeMaterial = new THREE.MeshPhongMaterial({
      color: 0x555555,
      map: planeTexture,
    });
    const plane = new THREE.Mesh(planeGeometry, planeMaterial);
    scene.add(plane);

    camera.position.z = 5;
    camera.position.y = -10;
    camera.lookAt(0, 5, 0);
    camera.up.set(0, 0, 1);

    const renderer = new WebgpuRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);
    const controls = new OrbitControls(camera, renderer.domElement);

    const light = new THREE.DirectionalLight(0xffffff, 10);
    const ambientLight = new THREE.AmbientLight(0x404040, 1);
    light.position.set(20000, 20000, 20000);
    light.target.position.set(0, 0, 0);
    scene.add(light);
    scene.add(ambientLight);

    renderer.setAnimationLoop(() => {
      renderer.render(scene, camera);
    });
  }
}
