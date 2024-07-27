import * as THREE from "three";
import fragShader from "./shaders/water.frag.glsl?raw";
import vertShader from "./shaders/water.vert.glsl?raw";
export class Water {
  constructor(
    public geometry: THREE.PlaneGeometry,
    public material: THREE.ShaderMaterial,
    public mesh: THREE.Mesh
  ) {}

  static startWater(WIDTH: number) {
    const waterPlaneGeometry = new THREE.PlaneGeometry(
      WIDTH / 5,
      WIDTH / 5,
      400,
      400
    );

    const waterShader = new THREE.ShaderMaterial({
      vertexShader: vertShader,
      fragmentShader: fragShader,
      blending: THREE.NormalBlending,
      transparent: true,
      opacity: 1.0,
      uniforms: {
        time: { value: 1.0 },
        directions: {
          value: [...makeDs()],
        },
        freq: { value: 0.1 },
        amplitude: { value: 1 },
        sunPosition: { value: new THREE.Vector3(1, 1, 1) },
      },
    });

    const mesh = new THREE.Mesh(waterPlaneGeometry, waterShader);

    return new Water(waterPlaneGeometry, waterShader, mesh);
  }

  setSunPosition(sunPosition: THREE.Vector3) {
    this.material.uniforms.sunPosition.value = sunPosition.clone().normalize();
  }

  tick(time: number) {
    this.material.uniforms.time.value = time / 1000;
  }
}

function makeDs() {
  return [...Array(10)].map((_, i) =>
    new THREE.Vector2(Math.random(), Math.random()).normalize()
  );
}
