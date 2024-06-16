import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";
import { GameWasmState } from "../pkg/game_state";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import boat from "./assets/ship.glb?url";
import { GUI } from "dat.gui";
import { UnrealBloomPass } from "three/addons/postprocessing/UnrealBloomPass.js";
import { EffectComposer } from "three/addons/postprocessing/EffectComposer.js";
import { RenderPass } from "three/addons/postprocessing/RenderPass.js";

export class Render3D {
  gui = new GUI();
  state = {
    boatScale: 0.06,
    boatPosition: [0, 0, 0.5],
    controlsEnabled: false,
    terrainColor: "#2b5232",
    waterColor: "#1d3d7d",
    skyColor: "#2fe6e6",
    bloomStrength: 1.5,
    bloomRadius: 0.4,
    bloomThreshold: 0.85,
  };
  gameState = GameWasmState.new();
  planeMesh = new THREE.Mesh();
  readonly PLANE_WIDTH = this.gameState.map_size();
  readonly SEGMENTS_DENSITY = 5;
  readonly PLANE_SEGMENTS = this.PLANE_WIDTH * this.SEGMENTS_DENSITY;

  private updateMesh() {
    const { geometry } = this.planeMesh;
    const arr = geometry.attributes.position.array;
    for (let x = 0; x < this.PLANE_SEGMENTS; x += 1) {
      for (let y = 0; y < this.PLANE_SEGMENTS; y += 1) {
        const i = (y * this.PLANE_SEGMENTS + x) * 3;
        let height =
          this.gameState.get_land_value(
            (x / this.PLANE_SEGMENTS) * this.PLANE_WIDTH - this.PLANE_WIDTH / 2,
            (y / this.PLANE_SEGMENTS) * this.PLANE_WIDTH - this.PLANE_WIDTH / 2
          ) ?? 0;
        height = height * 5;

        arr[i + 2] = height;
      }
    }
    geometry.attributes.position.needsUpdate = true;
    geometry.computeVertexNormals();
  }

  private saveState() {
    localStorage.setItem("state", JSON.stringify(this.state));
  }

  private loadState() {
    const state = localStorage.getItem("state");
    if (state) {
      this.state = { ...this.state, ...JSON.parse(state) };
    }
  }

  private addWaterColorControl(waterMaterial: THREE.MeshPhongMaterial) {
    this.gui.addColor(this.state, "waterColor").onChange((val) => {
      waterMaterial.color.set(val);
    });
  }

  private addTerrainColorControl(terrainMaterial: THREE.MeshLambertMaterial) {
    this.gui.addColor(this.state, "terrainColor").onChange((val) => {
      terrainMaterial.color.set(val);
    });
  }

  private addBloomControls(bloomPass: UnrealBloomPass) {
    this.gui.add(this.state, "bloomStrength", 0, 3).onChange((val) => {
      bloomPass.strength = val;
    });
    this.gui.add(this.state, "bloomRadius", 0, 1).onChange((val) => {
      bloomPass.radius = val;
    });
    this.gui.add(this.state, "bloomThreshold", 0, 1).onChange((val) => {
      bloomPass.threshold = val;
    });
  }

  async init() {
    this.loadState();
    setInterval(() => this.saveState(), 1_000);
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );

    const waterPlaneGeometry = new THREE.PlaneGeometry(
      this.PLANE_WIDTH,
      this.PLANE_WIDTH,
      1,
      1
    );
    const waterMaterial = new THREE.MeshPhongMaterial({
      color: this.state.waterColor,
      transparent: true,
      opacity: 0.9,
      shininess: 30,
      side: THREE.DoubleSide,
      fog: true,
    });

    this.addWaterColorControl(waterMaterial);
    const waterMesh = new THREE.Mesh(waterPlaneGeometry, waterMaterial);
    scene.add(waterMesh);

    const planeGeometry = new THREE.PlaneGeometry(
      this.PLANE_WIDTH,
      this.PLANE_WIDTH,
      this.PLANE_SEGMENTS - 1,
      this.PLANE_SEGMENTS - 1
    );

    scene.fog = new THREE.Fog(0x999999, 0, 100);

    const planeMaterial = new THREE.MeshLambertMaterial({
      color: this.state.terrainColor,
      fog: true,
    });
    this.addTerrainColorControl(planeMaterial);
    const plane = new THREE.Mesh(planeGeometry, planeMaterial);
    scene.add(plane);

    this.planeMesh = plane;
    this.updateMesh();

    camera.position.z = 5;
    camera.position.y = -10;
    camera.lookAt(0, 5, 0);
    camera.up.set(0, 0, 1);

    const renderer = new THREE.WebGLRenderer({});
    renderer.setClearColor(this.state.skyColor, 1);
    this.gui.addColor(this.state, "skyColor").onChange(() => {
      renderer.setClearColor(this.state.skyColor, 1);
    });
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);
    const orbit = new OrbitControls(camera, renderer.domElement);
    renderer.domElement.style.backgroundColor = "skyblue";

    const loader = new GLTFLoader();
    const controls = new TransformControls(camera, renderer.domElement);
    scene.add(controls);
    controls.addEventListener("mouseDown", () => {
      orbit.enabled = false;
    });
    controls.addEventListener("mouseUp", () => {
      orbit.enabled = true;
    });

    loader.load(boat, (_obj) => {
      const obj = _obj.scene;
      obj.scale.set(
        this.state.boatScale,
        this.state.boatScale,
        this.state.boatScale
      );
      console.log(_obj.animations);
      obj.rotation.set(Math.PI / 2, 0, 0);
      scene.add(obj);
      this.gui.add(this.state, "controlsEnabled").onChange(() => {
        if (this.state.controlsEnabled) {
          controls.attach(obj);
        } else {
          controls.detach();
        }
      });
      this.gui.add(this.state, "boatScale", 0.001, 0.5).onChange((val) => {
        obj.scale.set(val, val, val);
      });
    });

    this.makeSun(scene);
    const composer = new EffectComposer(renderer);
    const renderPass = new RenderPass(scene, camera);
    composer.addPass(renderPass);

    const bloomPass = new UnrealBloomPass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      this.state.bloomStrength,
      this.state.bloomRadius,
      this.state.bloomThreshold
    );
    composer.addPass(bloomPass);
    renderer.setAnimationLoop(() => {
      composer.render();
    });
    this.addBloomControls(bloomPass);
  }

  private makeSun(scene: THREE.Scene) {
    const sun = new THREE.SphereGeometry(30, 32, 32);
    const sunMaterial = new THREE.MeshLambertMaterial({
      color: 0xffff00,
      reflectivity: 0.0,
      refractionRatio: 0.0,
      emissive: 0xffff00,
      emissiveIntensity: 1,
      fog: false,
    });
    const sunMesh = new THREE.Mesh(sun, sunMaterial);
    const sunPosition = new THREE.Vector3(500, 0, 100);
    sunMesh.position.set(sunPosition.x, sunPosition.y, sunPosition.z);
    scene.add(sunMesh);
    const light = new THREE.DirectionalLight(0xffffff, 10);
    const ambientLight = new THREE.AmbientLight(0x404040, 30);
    light.position.set(sunPosition.x, sunPosition.y, sunPosition.z);
    light.target.position.set(0, 0, 0);
    scene.add(light);
    scene.add(ambientLight);
  }
}
