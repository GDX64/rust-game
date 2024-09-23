import * as THREE from "three";
import { GameWasmState, LocalClient, OnlineClient } from "../pkg/game_state";
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
import { PlayerActions } from "./PlayerActions";
import { LeaderBoards } from "./LeaderBoards";

function defaultState() {
  return {
    boatScale: 2.5,
    online: false,
    boatPosition: [0, 0, 0.5],
    controlsEnabled: false,
    terrainColor: "#2b5232",
    waterColor: "#1d3d7d",
    skyColor: "#f5d7a3",
    bloomStrength: 1.5,
    bloomRadius: 0.4,
    bloomThreshold: 0.85,
    redShift: false,
    windSpeed: 0,
    bloomEnabled: false,
    shootError: 0.1,
    showAxes: false,
    fastSimulation: false,
  };
}
export class Render3D {
  gui = new GUI();
  planeMesh = new THREE.Mesh();
  readonly pathLine = new THREE.Line(
    new THREE.BufferGeometry(),
    new THREE.LineBasicMaterial({ color: 0xffff00 })
  );
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.PerspectiveCamera(
    75,
    window.innerWidth / window.innerHeight,
    2,
    5_000
  );
  readonly PLANE_WIDTH;
  readonly SEGMENTS_DENSITY;
  readonly PLANE_SEGMENTS;
  readonly water;
  readonly shipsManager;
  readonly leaderboards;

  readonly terrain;
  readonly playerActions;
  readonly canvas;
  readonly cameraControls;

  constructor(
    public gameState: GameWasmState,
    public state: ReturnType<typeof defaultState>
  ) {
    this.PLANE_WIDTH = this.gameState.map_size();
    this.SEGMENTS_DENSITY = this.gameState.tile_size();
    this.PLANE_SEGMENTS = this.PLANE_WIDTH / this.SEGMENTS_DENSITY;
    this.water = Water.startWater(this.PLANE_WIDTH, this.gameState);
    this.shipsManager = new ShipsManager(
      this.gameState,
      this.scene,
      this.water,
      this.camera
    );
    this.terrain = Terrain.new(this.gameState);
    this.canvas = document.createElement("canvas");
    this.cameraControls = new CameraControl(
      this.camera,
      this.gameState.start_position()
    );
    this.playerActions = new PlayerActions(
      this.canvas,
      this.shipsManager,
      this.cameraControls,
      this.water,
      this.terrain
    );
    this.leaderboards = new LeaderBoards(this.gameState);

    //reset gui defaults
    this.gui.add(this.state, "fastSimulation");
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
      this.saveState();
      window.location.reload();
    });
    this.gameState.change_error(0);
    this.gui.add(this.state, "shootError", 0, 0.1).onChange((val) => {
      this.gameState.change_error(val);
    });
    this.gui.close();
  }

  private saveState() {
    localStorage.setItem("state", JSON.stringify(this.state));
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

  static async new(element: HTMLElement) {
    let state: any = localStorage.getItem("state");
    if (state) {
      state = { ...defaultState(), ...JSON.parse(state) };
    } else {
      state = defaultState();
    }
    const server = await this.startServer(state.online);
    const render = new Render3D(server, state);
    render.init(element);
    return render;
  }

  static async startServer(online: boolean) {
    if (online) {
      // const url = "https://archpelagus.glmachado.com/ws";
      //server id and user_name
      const url =
        "http://localhost:5000/ws?server_id=default&player_name=gdx64";
      const onlineData = OnlineClient.new(url);
      await onlineData.init();
      return GameWasmState.new_online(onlineData);
    }
    const localClient = LocalClient.new();
    await localClient.init();
    return GameWasmState.new_local(localClient);
  }

  async init(el: HTMLElement) {
    this.gameState.change_error(this.state.shootError);

    setInterval(() => this.saveState(), 1_000);
    this.playerActions.bindEvents();
    const scene = this.scene;

    this.addWaterColorControl();
    const sunPos = this.makeSun(scene);
    this.water.setSunPosition(sunPos);
    this.water.addToScene(scene);

    this.terrain.addToScene(scene);
    this.canvas.classList.add("main-canvas");
    el.appendChild(this.canvas);
    el.appendChild(this.terrain.minimap.mapCanvas);
    el.appendChild(this.leaderboards.canvas);

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
    const { composer } = this.addPostProcessing(renderer);

    let lastTime = 0;
    renderer.setAnimationLoop((_time) => {
      const time = _time / 1000;
      const dt = time - lastTime;
      if (!lastTime) {
        lastTime = time;
        return;
      }
      const N = this.state.fastSimulation ? 10 : 1;
      for (let i = 0; i < N; i++) {
        const gameTime = this.gameState.current_time + dt;
        this.gameState.tick(gameTime);
      }
      const gameTime = this.gameState.current_time;
      this.shipsManager.tick(gameTime);
      this.water.tick(gameTime, this.camera);
      this.terrain.tick(this.camera);
      this.playerActions.tick();
      this.cameraControls.tick(time);
      this.leaderboards.tick(dt);
      this.gameState.clear_flags();
      composer.render();
      lastTime = time;
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
    renderer.setPixelRatio(window.devicePixelRatio);
    composer.addPass(renderPass);
    // const gammaCorrection = new ShaderPass(GammaCorrectionShader);
    // composer.addPass(outline);
    // composer.addPass(gammaCorrection);

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
    const sunPosition = new THREE.Vector3(0, 4000, 500);
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
