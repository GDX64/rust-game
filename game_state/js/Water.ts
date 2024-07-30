import * as THREE from "three";
import fragShader from "./shaders/water.frag.glsl?raw";
import vertShader from "./shaders/water.vert.glsl?raw";

const FREQ_START = 0.05;
const WATER_DETAIL = 400;
export class Water {
  freq = FREQ_START;
  constructor(
    private material: THREE.ShaderMaterial,
    private mesh: THREE.Mesh
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

  changeWind(x: number, y: number) {
    const ds = makeDs(this.freq);
    ds[0].set(x / 1000, y / 1000);
    this.material.uniforms.directions.value = ds;
  }

  addControls(gui: dat.GUI) {
    const waterFolder = gui.addFolder("Water");
    waterFolder.add(this.material.uniforms.amplitude, "value", 0, 20);
    waterFolder.add(this, "freq", 0.01, 0.1).onChange(() => {
      this.material.uniforms.directions.value = makeDs(this.freq);
    });
    waterFolder
      .addColor(
        {
          scatter_color: this.material.uniforms.scatter_color.value.getHex(),
        },
        "scatter_color"
      )
      .onChange((color: string) => {
        this.material.uniforms.scatter_color.value = new THREE.Color(color);
      });
    waterFolder
      .add(
        {
          scatter_factor: this.material.uniforms.scatter_factor.value,
        },
        "scatter_factor",
        1,
        10
      )
      .onChange((value: number) => {
        this.material.uniforms.scatter_factor.value = value;
      });
    waterFolder
      .addColor(
        {
          water_color: this.material.uniforms.water_color.value.getHex(),
        },
        "water_color"
      )
      .onChange((color: string) => {
        this.material.uniforms.water_color.value = new THREE.Color(color);
      });
  }

  intersects(ray: THREE.Raycaster) {
    return ray.intersectObject(this.mesh);
  }

  addToScene(scene: THREE.Scene) {
    scene.add(this.mesh);
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
      WIDTH,
      WIDTH,
      WATER_DETAIL,
      WATER_DETAIL
    );

    const ds = makeDs(FREQ_START);

    const params: THREE.ShaderMaterialParameters = {
      vertexShader: vertShader,
      fragmentShader: fragShader,
      blending: THREE.NormalBlending,
      transparent: true,
      depthTest: true,
      opacity: 1.0,
      uniforms: {
        time: { value: 1.0 },
        directions: {
          value: ds,
        },
        scatter_color: { value: new THREE.Color("#00ff9d") },
        water_color: { value: new THREE.Color("#30b4ca") },
        scatter_factor: { value: 5.5 },
        amplitude: { value: 2 },
        sunPosition: { value: new THREE.Vector3(1, 1, 1) },
      },
    };

    const waterShader = new THREE.ShaderMaterial({
      ...params,
    });

    const mesh = new THREE.Mesh(waterPlaneGeometry, waterShader);

    return new Water(waterShader, mesh);
  }

  setSunPosition(sunPosition: THREE.Vector3) {
    this.material.uniforms.sunPosition.value = sunPosition.clone().normalize();
  }

  tick(time: number) {
    this.material.uniforms.time.value = time;
  }
}

function makeDs(freq: number) {
  return [...Array(8)].map((_, i) =>
    new THREE.Vector2(Math.random(), Math.random())
      .normalize()
      .multiplyScalar((i + 1) * freq)
  );
}
