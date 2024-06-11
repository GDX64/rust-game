import * as THREE from "three";
import WebgpuRenderer from "three/addons/renderers/webgpu/WebGPURenderer.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { WorldGen } from "../pkg/game_state";
import { OBJLoader } from "three/addons/loaders/ObjLoader.js";
import boat from "./assets_ignore/boat.obj?url";

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
        let height = world.get_land_value(x / 100, y / 100) * 40;
        height = Math.max(height, 0);

        const textureIndex = (y * PLANE_SEGMENTS + x) * 4;
        if (height > 0) {
          arr[i + 2] = height;
          textureMap[textureIndex] = 60;
          textureMap[textureIndex + 1] = 139;
          textureMap[textureIndex + 2] = 86;
          textureMap[textureIndex + 3] = 255;
        } else {
          textureMap[textureIndex] = 30;
          textureMap[textureIndex + 1] = 144;
          textureMap[textureIndex + 2] = 255;
          textureMap[textureIndex + 3] = 255;
        }
      }
    }
    planeGeometry.computeVertexNormals();

    const planeTexture = new THREE.DataTexture(
      textureMap,
      PLANE_SEGMENTS,
      PLANE_SEGMENTS
    );
    planeTexture.wrapS = THREE.ClampToEdgeWrapping;
    planeTexture.wrapT = THREE.ClampToEdgeWrapping;
    planeTexture.flipY = true;
    planeTexture.needsUpdate = true;

    const planeMaterial = new THREE.MeshLambertMaterial({
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
    renderer.domElement.style.backgroundColor = "skyblue";

    const light = new THREE.DirectionalLight(0xffffff, 10);
    const ambientLight = new THREE.AmbientLight(0x404040, 30);
    light.position.set(20000, 20000, 20000);
    light.target.position.set(0, 0, 0);
    scene.add(light);
    scene.add(ambientLight);
    const loader = new OBJLoader();
    loader.load(boat, (obj) => {
      const scale = 0.05;
      obj.rotateX(Math.PI / 2);
      obj.position.set(0, 0, 2);
      obj.scale.set(scale, scale, scale);
      scene.add(obj);
    });

    renderer.setAnimationLoop(() => {
      renderer.render(scene, camera);
    });
  }
}
