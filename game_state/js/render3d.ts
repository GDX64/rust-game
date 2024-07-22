import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";
import { GameWasmState, OnlineData } from "../pkg/game_state";
import { GUI } from "dat.gui";
import { UnrealBloomPass } from "three/addons/postprocessing/UnrealBloomPass.js";
import { EffectComposer } from "three/addons/postprocessing/EffectComposer.js";
import { RenderPass } from "three/addons/postprocessing/RenderPass.js";
import { ShaderPass } from "three/addons/postprocessing/ShaderPass.js";
import { ShipsManager } from "./ShipsManager";
import { Water } from "./Water";
import WebGPURenderer from "three/addons/renderers/webgpu/WebGPURenderer.js";
export class Render3D {
  gui = new GUI();
  state = {
    boatScale: 2.5,
    boatPosition: [0, 0, 0.5],
    controlsEnabled: false,
    terrainColor: "#2b5232",
    waterColor: "#1d3d7d",
    skyColor: "#2fe6e6",
    bloomStrength: 1.5,
    bloomRadius: 0.4,
    bloomThreshold: 0.85,
    redShift: false,
  };
  gameState = GameWasmState.new();
  planeMesh = new THREE.Mesh();
  readonly pathLine = new THREE.Line(
    new THREE.BufferGeometry(),
    new THREE.LineBasicMaterial({ color: 0xffff00 })
  );
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.PerspectiveCamera(
    75,
    window.innerWidth / window.innerHeight,
    0.1,
    this.gameState.map_size() / 2
  );
  readonly rayCaster = new THREE.Raycaster();
  readonly mouse = new THREE.Vector2(0, 0);
  readonly PLANE_WIDTH = this.gameState.map_size();
  readonly SEGMENTS_DENSITY = this.gameState.tile_size();
  readonly PLANE_SEGMENTS = this.PLANE_WIDTH / this.SEGMENTS_DENSITY;
  readonly shipsManager = new ShipsManager(
    this.gameState,
    this.state.boatScale,
    this.scene,
    this.camera
  );

  waterMesh = new THREE.Mesh();

  private updateMesh() {
    const { geometry } = this.planeMesh;
    const arr = geometry.attributes.position.array;
    for (let x = 0; x < this.PLANE_SEGMENTS; x += 1) {
      for (let y = 0; y < this.PLANE_SEGMENTS; y += 1) {
        const i = (y * this.PLANE_SEGMENTS + x) * 3;
        const yProportion = y / this.PLANE_SEGMENTS;
        let height =
          this.gameState.get_land_value(
            (x / this.PLANE_SEGMENTS) * this.PLANE_WIDTH - this.PLANE_WIDTH / 2,
            (0.5 - yProportion) * this.PLANE_WIDTH
          ) ?? 0;
        height = height * 500;

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
    waterMaterial.color.set(this.state.waterColor);
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

  private onMouseClick(event: PointerEvent) {
    this.mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
    this.mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;
    this.rayCaster.setFromCamera(this.mouse, this.camera);
    const intersects = this.rayCaster.intersectObject(this.waterMesh);
    if (intersects.length > 0) {
      const [x, y] = intersects[0].point.toArray();
      if (event.button === 2) {
        this.shipsManager.moveShip(x, y);
      } else {
        this.shipsManager.shoot(x, y);
      }
    }
  }

  private async startRemoteServer() {
    const url = "http://game.glmachado.com:5000/ws";
    // const url = "http://localhost:5000/ws";
    const ws = new WebSocket(url);
    ws.binaryType = "arraybuffer";
    const onlineData = OnlineData.new(url);
    await onlineData.init();
    this.gameState.start_online(onlineData);
  }

  async startServer() {
    // this.gameState.start_local_server();
    await this.startRemoteServer();
    this.gameState.add_bot();
    this.gameState.add_bot();
  }

  async init() {
    await this.startServer();
    this.loadState();
    setInterval(() => this.saveState(), 1_000);
    const camera = this.camera;
    const scene = this.scene;

    const { waterMaterial, waterMesh } = Water.startWater(this.PLANE_WIDTH);

    this.addWaterColorControl(waterMaterial);
    this.waterMesh = waterMesh;
    scene.add(waterMesh);

    const planeGeometry = new THREE.PlaneGeometry(
      this.PLANE_WIDTH,
      this.PLANE_WIDTH,
      this.PLANE_SEGMENTS - 1,
      this.PLANE_SEGMENTS - 1
    );

    // scene.fog = new THREE.Fog(0x999999, 0, 100);

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

    const renderer = new WebGPURenderer();
    renderer.setClearColor(new THREE.Color(this.state.skyColor), 1);
    this.gui.addColor(this.state, "skyColor").onChange(() => {
      renderer.setClearColor(new THREE.Color(this.state.skyColor), 1);
    });
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);
    const orbit = new OrbitControls(camera, renderer.domElement);

    const controls = new TransformControls(camera, renderer.domElement);
    scene.add(controls);
    controls.addEventListener("mouseDown", () => {
      orbit.enabled = false;
    });
    controls.addEventListener("mouseUp", () => {
      orbit.enabled = true;
    });
    this.makeSun(scene);
    renderer.setAnimationLoop(() => {
      renderer.render(scene, camera);
      this.gameState.tick();
      this.shipsManager.tick();
    });

    window.addEventListener("pointerdown", (event) => this.onMouseClick(event));
  }

  private addPostProcessing(renderer: THREE.WebGLRenderer) {
    const composer = new EffectComposer(renderer);
    const renderPass = new RenderPass(this.scene, this.camera);
    renderer.setPixelRatio(window.devicePixelRatio);
    composer.addPass(renderPass);

    const bloomPass = new UnrealBloomPass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      this.state.bloomStrength,
      this.state.bloomRadius,
      this.state.bloomThreshold
    );
    composer.addPass(bloomPass);
    const redShift = redShiftEffect();
    if (this.state.redShift) {
      composer.addPass(redShift);
    }
    this.gui.add(this.state, "redShift").onChange((val) => {
      if (val) {
        composer.addPass(redShift);
      } else {
        composer.removePass(redShift);
      }
    });
    renderer.setAnimationLoop(() => {
      composer.render();
      this.gameState.tick();
      this.shipsManager.tick();
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
    const sunPosition = new THREE.Vector3(
      this.gameState.map_size(),
      0,
      this.gameState.map_size() / 2
    );
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

function redShiftEffect() {
  // Custom shader for red shift
  const redShiftShader = {
    uniforms: {
      tDiffuse: { value: null },
    },
    vertexShader: `
        varying vec2 vUv;
        void main() {
            vUv = uv;
            gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
    `,
    fragmentShader: `
        uniform sampler2D tDiffuse;
        varying vec2 vUv;
        void main() {
            vec4 color = texture2D(tDiffuse, vUv);
            color = vec4(min(1.0, color.r + 0.2), color.g, color.b, color.a); // Increase red channel
            gl_FragColor = color;
        }
    `,
  };

  const shaderPass = new ShaderPass(redShiftShader);
  shaderPass.renderToScreen = true;
  return shaderPass;
}
