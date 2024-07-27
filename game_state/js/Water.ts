import * as THREE from "three";
import fragShader from "./shaders/water.frag.glsl?raw";
import vertShader from "./shaders/water.vert.glsl?raw";
export class Water {
  constructor(
    public geometry: THREE.PlaneGeometry,
    public material: THREE.ShaderMaterial,
    public mesh: THREE.Mesh
  ) {}

  private getDirections(): THREE.Vector2[] {
    return this.material.uniforms.directions.value;
  }

  private freq(): number {
    return this.material.uniforms.freq.value;
  }

  private amplitude(): number {
    return this.material.uniforms.amplitude.value;
  }

  private time(): number {
    return this.material.uniforms.time.value;
  }

  calcElevationAt(x: number, y: number) {
    let acc = 0;
    const freq = this.freq();
    const time = this.time();
    const pos = new THREE.Vector2(x, y);
    this.getDirections().forEach((d, i) => {
      const harmonic = i + 1;
      const angle = d.dot(pos) * harmonic * freq + time;
      acc += Math.sin(angle) / harmonic / harmonic;
    });
    return acc * this.amplitude();
  }

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
