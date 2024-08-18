import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import vertexShader from "./shaders/explosion.vert.glsl?raw";
import fragmentShader from "./shaders/explosion.frag.glsl?raw";
import { RenderOrder } from "./RenderOrder";
import { ExplosionData } from "./RustWorldTypes";
import explosionImage from "./assets/explosion.png";
import { ExplosionKind } from "../pkg/game_state";

const PARTICLES = 50;

export class ExplosionManager {
  explosions: Map<number, Explosion> = new Map();
  explosionPool: Explosion[] = [];
  group = new THREE.Group();
  currentTime = 0;
  constructor(scene: THREE.Scene) {
    scene.add(this.group);
    this.group.renderOrder = RenderOrder.PARTICLES;
  }

  explodeData(data: ExplosionData, color: THREE.Color) {
    const position = new THREE.Vector3(data.position[0], data.position[1], 0);
    if (this.explosions.has(data.id)) {
      return;
    }
    let velocity;
    let size;
    if (data.kind === ExplosionKind.Bullet) {
      velocity = 7;
      size = 0.7;
    } else if (data.kind === ExplosionKind.Ship) {
      velocity = 30;
      size = 3;
    } else {
      velocity = 3;
      size = 0.3;
    }

    const explosion = this.explosionPool.pop() ?? new Explosion();
    explosion.setParams({
      size,
      velocity,
      position,
      id: data.id,
      color,
    });
    explosion.addToScene(this.group);
    this.explosions.set(data.id, explosion);
  }

  tick(time: number) {
    const delta = time - this.currentTime;
    this.currentTime = time;
    this.explosions.forEach((explosion) => {
      explosion.tick(delta);
      if (explosion.isFinished) {
        this.explosions.delete(explosion.id);
        this.explosionPool.push(explosion);
        this.group.remove(explosion.points);
      }
    });
  }
}

export class Explosion {
  isFinished = false;
  private v = 15;
  points: THREE.Points<THREE.BufferGeometry, THREE.ShaderMaterial>;
  private particlesPosition: THREE.Vector3[] = [];
  private timeToLive = 0;
  private t = 0;
  public id: number = -1;
  constructor() {
    const { points } = Explosion.makePoints();
    this.points = points;
    this.particlesPosition = new Array(PARTICLES)
      .fill(0)
      .map(() => new THREE.Vector3());
  }

  setParams({
    size = 1,
    timeToLive = 3,
    position = new THREE.Vector3(),
    id = 0,
    color = new THREE.Color(0xffff00),
    velocity = this.v,
  } = {}) {
    this.points.position.set(position.x, position.y, position.z);
    this.particlesPosition.forEach((particle) => {
      particle.set(0, 0, 0);
    });
    this.id = id;
    this.isFinished = false;
    this.t = 0;
    this.v = velocity;
    this.timeToLive = timeToLive;
    this.points.material.uniforms.color.value = color;
    this.points.material.uniforms.pointMultiplier.value = size;
    this.randomizeSpeed();
    this.tick(0);
  }

  addToScene(group: THREE.Group) {
    this.t = 0;
    group.add(this.points);
  }

  private randomizeSpeed() {
    const speed = this.points.geometry.attributes.speed;
    speed.needsUpdate = true;
    const particle = new THREE.Vector3();
    for (let i = 0; i < PARTICLES; i++) {
      particle.set(
        Math.random() * 1 - 0.5,
        Math.random() * 1 - 0.5,
        Math.random() * 0.5
      );
      particle.normalize().multiplyScalar(this.v * Math.random());
      const index = i * 3;
      speed.array[index] = particle.x;
      speed.array[index + 1] = particle.y;
      speed.array[index + 2] = particle.z;
    }
  }

  tick(dt: number) {
    this.t += dt;
    const animationPercent = this.t / this.timeToLive;
    this.points.material.uniforms.progress.value = animationPercent;
    this.points.material.uniforms.time.value = this.t;
    if (this.t > this.timeToLive) {
      this.isFinished = true;
    }
  }

  static testRenderer() {
    const camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );
    camera.position.set(0, 0, 100);
    const scene = new THREE.Scene();
    const renderer = new THREE.WebGLRenderer();

    const orbit = new OrbitControls(camera, renderer.domElement);
    document.body.appendChild(renderer.domElement);

    const explosionManager = new ExplosionManager(scene);
    document.addEventListener("click", () => {
      // explosionManager.explodeAt(
      //   new THREE.Vector3(0, 0, 0),
      //   Math.random() * 1000,
      //   new THREE.Color(Math.random() * 0xffffff)
      // );
    });

    const light = new THREE.DirectionalLight(0xffffff, 100);
    light.position.set(50, 50, 50);
    scene.add(light);
    renderer.setClearColor(0x000000, 1);
    renderer.setSize(window.innerWidth, window.innerHeight);

    renderer.setAnimationLoop((time) => {
      renderer.render(scene, camera);
      orbit.update();
      explosionManager.tick(time / 1000);
    });
  }

  static makePoints() {
    const geometry = new THREE.BufferGeometry();
    const vertices = new Float32Array(PARTICLES * 3);
    // const colors = new Float32Array(particles * 3);
    geometry.setAttribute("speed", new THREE.BufferAttribute(vertices, 3));
    geometry.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(vertices), 3)
    );
    // geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    const texture = new THREE.TextureLoader().load(explosionImage);
    const pointMaterial = new THREE.ShaderMaterial({
      fragmentShader,
      vertexShader,
      uniforms: {
        diffuseTexture: { value: texture },
        time: { value: 0 },
        progress: { value: 0 },
        color: { value: new THREE.Color(0xffff00) },
        pointMultiplier: { value: 1 },
      },
      blending: THREE.AdditiveBlending,
      transparent: true,
      depthTest: false,
    });
    const points = new THREE.Points(geometry, pointMaterial);
    return { points };
  }
}
