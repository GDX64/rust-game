import * as THREE from "three";
import fragShader from "./shaders/water.frag.glsl?raw";
import vertShader from "./shaders/water.vert.glsl?raw";
export class Water {
  constructor() {}

  static startWater(WIDTH: number) {
    const waterPlaneGeometry = new THREE.PlaneGeometry(WIDTH, WIDTH, 300, 300);

    const waterMaterial = new THREE.MeshPhongMaterial({
      color: 0x0000ff,
      transparent: true,
      opacity: 0.9,
      shininess: 30,
      side: THREE.DoubleSide,
      fog: true,
    });

    const waterShader = new THREE.ShaderMaterial({
      vertexShader: vertShader,
      fragmentShader: fragShader,
      uniforms: {
        time: { value: 1.0 },
        resolution: { value: new THREE.Vector2() },
        // cameraPosition: { value: new THREE.Vector3() },
      },
    });

    const waterMesh = new THREE.Mesh(waterPlaneGeometry, waterShader);

    return { waterPlaneGeometry, waterMaterial, waterMesh, waterShader };
  }
}
