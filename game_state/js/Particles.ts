import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

export type ExplosionData = {
  position: [number, number];
  id: number;
};

export class ExplosionManager {
  explosions: Map<number, Explosion> = new Map();
  constructor(public scene: THREE.Scene) {}

  explodeData(data: ExplosionData) {
    this.explodeAt(
      new THREE.Vector3(data.position[0], data.position[1], 0),
      data.id
    );
  }

  explodeAt(position: THREE.Vector3, id: number) {
    if (this.explosions.has(id)) {
      return;
    }
    const explosion = new Explosion({
      particles: 1_000,
      size: 1,
      position,
      id,
    });
    explosion.addToScene(this.scene);
    this.explosions.set(id, explosion);
  }

  tick(dt: number) {
    this.explosions.forEach((explosion) => {
      explosion.tick(dt);
      if (explosion.isFinished) {
        this.explosions.delete(explosion.id);
      }
    });
  }
}

export class Explosion {
  isFinished = false;
  private v = 30;
  private points: THREE.Points;
  private particlesPosition: THREE.Vector3[];
  private particlesSpeed: THREE.Vector3[];
  private timeToLive;
  private t = 0;
  public readonly id: number;
  private scene: null | THREE.Scene = null;
  constructor({
    particles = 1000,
    size = 1,
    timeToLive = 2,
    position = new THREE.Vector3(),
    id = 0,
  } = {}) {
    const { points } = Explosion.makePoints(particles, size);
    points.position.set(position.x, position.y, position.z);
    this.id = id;
    this.points = points;
    this.particlesPosition = new Array(particles)
      .fill(0)
      .map(() => new THREE.Vector3());
    this.particlesSpeed = new Array(particles).fill(0).map(() => {
      return new THREE.Vector3();
    });
    this.randomizeSpeed();
    this.timeToLive = timeToLive;
  }

  addToScene(scene: THREE.Scene) {
    this.t = 0;
    this.scene = scene;
    scene.add(this.points);
  }

  private randomizeSpeed() {
    this.particlesSpeed.forEach((particle) => {
      particle.set(
        Math.random() * 1 - 0.5,
        Math.random() * 1 - 0.5,
        Math.random() * 1 - 0.5
      );
      particle.normalize().multiplyScalar(this.v * Math.random());
    });
  }

  tick(dt: number) {
    const position = this.points.geometry.attributes.position;
    position.needsUpdate = true;

    this.t += dt;
    const animationPercent = this.t / this.timeToLive;

    this.particlesPosition.forEach((particle, i) => {
      particle.add(this.particlesSpeed[i].clone().multiplyScalar(dt));
      const vecIndex = i * 3;
      position.array[vecIndex] = particle.x;
      position.array[vecIndex + 1] = particle.y;
      position.array[vecIndex + 2] = particle.z;
    });
    if (this.points.material instanceof THREE.PointsMaterial) {
      this.points.material.opacity = 1 - animationPercent;
    }
    if (this.t > this.timeToLive) {
      this.scene?.remove(this.points);
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
    camera.position.set(50, 0, 50);
    const scene = new THREE.Scene();
    const renderer = new THREE.WebGLRenderer();

    const orbit = new OrbitControls(camera, renderer.domElement);
    document.body.appendChild(renderer.domElement);

    const explosionManager = new ExplosionManager(scene);
    document.addEventListener("click", () => {
      explosionManager.explodeAt(
        new THREE.Vector3(0, 0, 0),
        Math.random() * 10000
      );
    });

    const light = new THREE.DirectionalLight(0xffffff, 100);
    light.position.set(50, 50, 50);
    scene.add(light);
    renderer.setClearColor(0xffffaa, 1);
    renderer.setSize(window.innerWidth, window.innerHeight);

    renderer.setAnimationLoop(() => {
      renderer.render(scene, camera);
      orbit.update();
      explosionManager.tick(0.016);
    });
  }

  static makePoints(particles: number = 1000, size: number = 1) {
    const geometry = new THREE.BufferGeometry();
    const vertices = new Float32Array(particles * 3);
    // const colors = new Float32Array(particles * 3);
    geometry.setAttribute("position", new THREE.BufferAttribute(vertices, 3));
    // geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    const pointMaterial = new THREE.PointsMaterial({
      color: 0xffff00,
      // map: texture,
      blending: THREE.NormalBlending,
      size,
      depthTest: true,
      transparent: true,
      opacity: 1,
      // vertexColors: true,
    });
    const points = new THREE.Points(geometry, pointMaterial);
    return { points };
  }
}
