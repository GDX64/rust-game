import * as THREE from "three";
import fragShader from "./shaders/water.frag.glsl?raw";
import vertShader from "./shaders/water.vert.glsl?raw";
import { RenderOrder } from "./RenderOrder";

const FREQ_START = 0.05;
const WIDTH = 2_000;
const WATER_DETAIL = 400;
export class Water {
  freq = FREQ_START;

  constructor(
    private material: THREE.ShaderMaterial,
    private simpleMaterial: THREE.ShaderMaterial,
    private simpleMesh: THREE.Mesh,
    private intersectionPlane: THREE.Mesh,
    private waterGroup: THREE.Group
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

  // changeWind(x: number, y: number) {
  //   const ds = makeDs(this.freq);
  //   ds[0].set(x / 1000, y / 1000);
  //   this.material.uniforms.directions.value = ds;
  // }

  addControls(gui: dat.GUI) {
    // const waterFolder = gui.addFolder("Water");
    // waterFolder.add(this.material.uniforms.amplitude, "value", 0, 20);
    // waterFolder.add(this, "freq", 0.01, 0.1).onChange(() => {
    //   this.material.uniforms.directions.value = makeDs(this.freq);
    // });
    // waterFolder
    //   .addColor(
    //     {
    //       scatter_color: this.material.uniforms.scatter_color.value.getHex(),
    //     },
    //     "scatter_color"
    //   )
    //   .onChange((color: string) => {
    //     this.material.uniforms.scatter_color.value = new THREE.Color(color);
    //   });
    // waterFolder
    //   .add(
    //     {
    //       scatter_factor: this.material.uniforms.scatter_factor.value,
    //     },
    //     "scatter_factor",
    //     1,
    //     10
    //   )
    //   .onChange((value: number) => {
    //     this.material.uniforms.scatter_factor.value = value;
    //   });
    // waterFolder
    //   .addColor(
    //     {
    //       water_color: this.material.uniforms.water_color.value.getHex(),
    //     },
    //     "water_color"
    //   )
    //   .onChange((color: string) => {
    //     this.material.uniforms.water_color.value = new THREE.Color(color);
    //     this.simpleMaterial.uniforms.water_color.value = new THREE.Color(color);
    //   });
  }

  intersects(ray: THREE.Raycaster) {
    return ray.intersectObject(this.intersectionPlane);
  }

  addToScene(scene: THREE.Scene) {
    scene.add(this.waterGroup);
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

  static startWater(width: number) {
    const waterPlaneGeometry = new THREE.PlaneGeometry(
      WIDTH,
      WIDTH,
      WATER_DETAIL,
      WATER_DETAIL
    );

    const waterShader = waterCustomShader(makeDs(FREQ_START), false);

    const completeWater = new THREE.Mesh(waterPlaneGeometry, waterShader);
    completeWater.renderOrder = RenderOrder.Water;

    const simpleWaterShader = waterCustomShader(makeDs(0), true);

    const simplePlane = new THREE.Mesh(
      new THREE.PlaneGeometry(width * 2, width * 2, 1, 1),
      // waterPlaneGeometry,
      simpleWaterShader
    );

    completeWater.visible = true;
    simplePlane.visible = true;

    simplePlane.renderOrder = RenderOrder.Water;

    const testPlane = new THREE.PlaneGeometry(WIDTH, WIDTH);
    const testMaterial = new THREE.MeshLambertMaterial({
      color: 0x000000,
      colorWrite: false,
      depthTest: false,
      depthWrite: false,
      stencilWrite: true,
      stencilFunc: THREE.AlwaysStencilFunc,
      stencilZPass: THREE.ReplaceStencilOp,
      stencilRef: 1,
    });
    const stencilPlane = new THREE.Mesh(testPlane, testMaterial);
    stencilPlane.position.set(0, 0, -3);

    const watergroup = new THREE.Group();
    watergroup.add(completeWater);
    watergroup.add(stencilPlane);
    watergroup.add(simplePlane);

    const intersectionPlane = new THREE.Mesh(
      new THREE.PlaneGeometry(width, width, 1, 1),
      new THREE.MeshBasicMaterial({
        color: 0x000000,
        visible: false,
      })
    );

    return new Water(
      waterShader,
      simpleWaterShader,
      simplePlane,
      intersectionPlane,
      watergroup
    );
  }

  setSunPosition(sunPosition: THREE.Vector3) {
    this.material.uniforms.sunPosition.value = sunPosition.clone().normalize();
    this.simpleMaterial.uniforms.sunPosition.value = sunPosition
      .clone()
      .normalize();
  }

  tick(time: number, camera: THREE.Camera) {
    this.material.uniforms.time.value = time;
    this.simpleMaterial.uniforms.time.value = time;
    this.waterGroup.position.set(camera.position.x, camera.position.y, 0);
  }
}

function waterCustomShader(ds: THREE.Vec2[], stencil: boolean) {
  return new THREE.ShaderMaterial({
    vertexShader: vertShader,
    fragmentShader: fragShader,
    blending: THREE.NormalBlending,
    transparent: true,
    depthWrite: false,
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
    premultipliedAlpha: false,
    stencilWrite: stencil,
    stencilFunc: THREE.EqualStencilFunc,
    stencilRef: 0,
  });
}

function makeDs(freq: number) {
  return [...Array(8)].map((_, i) =>
    new THREE.Vector2(Math.random(), Math.random())
      .normalize()
      .multiplyScalar((i + 1) * freq)
  );
}
