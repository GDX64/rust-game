import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

export class Explosion {
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
    const SphereGeometry = new THREE.BoxGeometry(10, 10, 10);
    const SphereMaterial = new THREE.MeshPhongMaterial({
      color: 0xff0000,
      reflectivity: 0.5,
      shininess: 100,
    });
    const orbit = new OrbitControls(camera, renderer.domElement);
    const sun = new THREE.Mesh(SphereGeometry, SphereMaterial);
    scene.add(sun);
    scene;
    document.body.appendChild(renderer.domElement);
    const light = new THREE.DirectionalLight(0xffffff, 100);
    light.position.set(50, 50, 50);
    scene.add(light);
    renderer.setClearColor(0xffffaa, 1);
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.setAnimationLoop(() => {
      renderer.render(scene, camera);
      orbit.update();
    });
  }
}
