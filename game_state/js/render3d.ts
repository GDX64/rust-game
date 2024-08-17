import * as THREE from "three";
import { GameWasmState, OnlineClient } from "../pkg/game_state";
import { GUI } from "dat.gui";
import { UnrealBloomPass } from "three/addons/postprocessing/UnrealBloomPass.js";
import { EffectComposer } from "three/addons/postprocessing/EffectComposer.js";
import { RenderPass } from "three/addons/postprocessing/RenderPass.js";
import { ShaderPass } from "three/addons/postprocessing/ShaderPass.js";
import { OutlinePass } from "three/addons/postprocessing/OutlinePass.js";
import { ShipsManager } from "./ShipsManager";
import { Water } from "./Water";
import { CameraControl } from "./CameraControl";
import { Terrain } from "./Terrain";
import { GammaCorrectionShader } from "three/addons/shaders/GammaCorrectionShader.js";
import { PlayerActions } from "./PlayerActions";

function defaultState() {
  return {
    boatScale: 2.5,
    online: false,
    boatPosition: [0, 0, 0.5],
    controlsEnabled: false,
    terrainColor: "#2b5232",
    waterColor: "#1d3d7d",
    skyColor: "#2fe6e6",
    bloomStrength: 1.5,
    bloomRadius: 0.4,
    bloomThreshold: 0.85,
    redShift: false,
    windSpeed: 0,
    bloomEnabled: false,
    shootError: 0.01,
    showAxes: false,
  };
}
export class Render3D {
  gui = new GUI();
  state = defaultState();
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
    5_000
  );
  readonly PLANE_WIDTH = this.gameState.map_size();
  readonly SEGMENTS_DENSITY = this.gameState.tile_size();
  readonly PLANE_SEGMENTS = this.PLANE_WIDTH / this.SEGMENTS_DENSITY;
  readonly water = Water.startWater(this.PLANE_WIDTH);
  readonly shipsManager = new ShipsManager(
    this.gameState,
    this.scene,
    this.water
  );

  outline = new OutlinePass(new THREE.Vector2(), this.scene, this.camera);

  readonly terrain = Terrain.new(this.gameState);
  readonly playerActions;
  readonly canvas;
  readonly cameraControls;

  constructor() {
    this.loadState();
    this.canvas = document.createElement("canvas");
    this.cameraControls = new CameraControl(this.camera);
    this.playerActions = new PlayerActions(
      this.canvas,
      this.shipsManager,
      this.cameraControls,
      this.water
    );

    //reset gui defaults
    this.gui.add(
      {
        reset: () => {
          Object.assign(this.state, defaultState());
          this.gui.updateDisplay();
        },
      },
      "reset"
    );
    this.gui.add(
      {
        addBot: () => {
          this.gameState.add_bot();
        },
      },
      "addBot"
    );
    this.gui.add(
      {
        removeBot: () => {
          this.gameState.remove_bot();
        },
      },
      "removeBot"
    );
    this.gui.add(this.state, "online").onChange((val) => {
      if (val) {
        this.startRemoteServer();
      } else {
        this.gameState.start_local_server();
      }
    });
    this.gameState.change_error(0);
    this.gui.add(this.state, "shootError", 0, 0.1).onChange((val) => {
      this.gameState.change_error(val);
    });
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

  private addWaterColorControl() {
    this.water.addControls(this.gui);
  }

  private addBloomControls(bloomPass: UnrealBloomPass) {
    bloomPass.enabled = this.state.bloomEnabled;
    const bloomFolder = this.gui.addFolder("Bloom");
    bloomFolder.add(this.state, "bloomStrength", 0, 3).onChange((val) => {
      bloomPass.strength = val;
    });
    bloomFolder.add(this.state, "bloomRadius", 0, 1).onChange((val) => {
      bloomPass.radius = val;
    });
    bloomFolder.add(this.state, "bloomThreshold", 0, 1).onChange((val) => {
      bloomPass.threshold = val;
    });
    bloomFolder.add(this.state, "bloomEnabled").onChange((val) => {
      if (val) {
        bloomPass.enabled = true;
      } else {
        bloomPass.enabled = false;
      }
    });
  }

  private async startRemoteServer() {
    const url = "https://game.glmachado.com/ws";
    // const url = "http://localhost:5000/ws";
    const onlineData = OnlineClient.new(url);
    await onlineData.init();
    this.gameState.start_online(onlineData);
  }

  async init() {
    if (this.state.online) {
      await this.startRemoteServer();
    } else {
      this.gameState.start_local_server();
    }
    setInterval(() => this.saveState(), 1_000);
    document.body.appendChild(this.canvas);
    this.playerActions.bindEvents();
    const scene = this.scene;

    this.addWaterColorControl();
    const sunPos = this.makeSun(scene);
    this.water.setSunPosition(sunPos);
    this.water.addToScene(scene);

    this.terrain.addToScene(scene);

    const renderer = new THREE.WebGLRenderer({
      antialias: true,
      canvas: this.canvas,
      stencil: true,
    });

    const fog = new THREE.Fog(0x999999, 0, 4000);
    this.scene.fog = fog;

    renderer.setClearColor(new THREE.Color(this.state.skyColor), 1);
    this.gui.addColor(this.state, "skyColor").onChange(() => {
      renderer.setClearColor(new THREE.Color(this.state.skyColor), 1);
    });
    renderer.setSize(window.innerWidth, window.innerHeight);

    this.cameraControls.addListeners();
    const { composer, outline } = this.addPostProcessing(renderer);

    this.outline = outline;

    renderer.setAnimationLoop((_time) => {
      const time = _time / 1000;
      this.playerActions.tick();
      this.gameState.tick(time);
      this.shipsManager.tick(time);
      this.water.tick(time, this.camera);
      this.cameraControls.tick(time);
      // composer.render();
      renderer.render(scene, this.camera);
    });

    this.shipsManager.selected$.subscribe((ship) => {
      this.outline.selectedObjects = [ship];
      this.outline.selectedObjects;
    });

    const helpersFolder = this.gui.addFolder("Helpers");

    this.playerActions.showHelper(this.state.showAxes);
    helpersFolder.add(this.state, "showAxes").onChange((val) => {
      this.playerActions.showHelper(val);
    });
  }

  private addPostProcessing(renderer: THREE.WebGLRenderer) {
    const renderTarget = new THREE.WebGLRenderTarget(
      window.innerWidth,
      window.innerHeight,
      {
        stencilBuffer: true,
      }
    );
    const composer = new EffectComposer(renderer, renderTarget);
    // composer.stencilBuffer = true;
    const renderPass = new RenderPass(this.scene, this.camera);
    const outline = new OutlinePass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      this.scene,
      this.camera
    );
    const gammaCorrection = new ShaderPass(GammaCorrectionShader);
    renderer.setPixelRatio(window.devicePixelRatio);
    composer.addPass(renderPass);
    composer.addPass(outline);
    composer.addPass(gammaCorrection);

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
    this.addBloomControls(bloomPass);
    return { composer, outline };
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
    const sunPosition = new THREE.Vector3(4000, 0, 1500);
    sunMesh.position.set(sunPosition.x, sunPosition.y, sunPosition.z);
    scene.add(sunMesh);
    const light = new THREE.DirectionalLight(0xffffff, 10);
    const ambientLight = new THREE.AmbientLight(0x404040, 30);
    light.position.set(sunPosition.x, sunPosition.y, sunPosition.z);
    light.target.position.set(0, 0, 0);
    scene.add(light);
    scene.add(ambientLight);
    return sunPosition;
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
