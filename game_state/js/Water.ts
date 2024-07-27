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

  private amplitude(): number {
    return this.material.uniforms.amplitude.value;
  }

  private time(): number {
    return this.material.uniforms.time.value;
  }

  calcElevationAt(x: number, y: number) {
    let acc = 0;
    let derivative = new THREE.Vector2(0, 0);
    const time = this.time();
    const pos = new THREE.Vector2(x, y);
    this.getDirections().forEach((d, i) => {
      const harmonic = (i + 1) ** 2;
      const angle = d.dot(pos) + time;
      acc += Math.sin(angle) / harmonic;

      const derivativeLength = Math.cos(angle) / harmonic;
      derivative.x += d.x * derivativeLength;
      derivative.y += d.y * derivativeLength;
    });
    derivative.multiplyScalar(this.amplitude());
    const normal = new THREE.Vector3(1, 0, derivative.x)
      .cross(new THREE.Vector3(0, 1, derivative.y))
      .normalize();
    return [acc * this.amplitude(), normal] as const;
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
        amplitude: { value: 1.5 },
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

function makeDs(freq = 0.1) {
  return [...Array(8)].map((_, i) =>
    new THREE.Vector2(Math.random(), Math.random())
      .normalize()
      .multiplyScalar((i + 1) * freq)
  );
}
