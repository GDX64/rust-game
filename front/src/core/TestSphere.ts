import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/Addons.js";
import textureURL from "../assets/earth_texture.jpg";

export class TestSphere {
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

    const geometry = new THREE.SphereGeometry(5, 64, 64);

    const texture = new THREE.TextureLoader().load(textureURL);
    const mesh = new THREE.Mesh(
      geometry,
      new THREE.MeshLambertMaterial({
        color: 0xffffff,
        flatShading: false,
        map: texture,
      })
    );

    scene.add(mesh);

    const orbit = new OrbitControls(camera, renderer.domElement);
    document.body.appendChild(renderer.domElement);

    const light = new THREE.DirectionalLight(0xffffff, 10);
    const ambientLight = new THREE.AmbientLight(0x404040, 30);
    light.position.set(50, 50, 50);
    scene.add(light);
    scene.add(ambientLight);
    renderer.setClearColor(0x000000, 1);
    renderer.setSize(window.innerWidth, window.innerHeight);

    renderer.setAnimationLoop((time) => {
      renderer.render(scene, camera);
      orbit.update();
    });
  }
}
