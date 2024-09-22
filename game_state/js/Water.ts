import * as THREE from "three";
import fragShader from "./shaders/water.frag.glsl?raw";
import vertShader from "./shaders/water.vert.glsl?raw";
import { RenderOrder } from "./RenderOrder";
import normalMap from "./assets/water_normals.jpg";
import { GameWasmState } from "../pkg/game_state";

const FREQ_START = 0.025;
const WIDTH = 5_000;
const WATER_DETAIL = 100;
const DIR = 1;

const roundingFactor = 1;
const round = (x: number) => Math.floor(x / roundingFactor) * roundingFactor;

export class Water {
  freq = FREQ_START;

  constructor(
    private material: THREE.ShaderMaterial,
    private simpleMaterial: THREE.ShaderMaterial,
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
    const waterFolder = gui.addFolder("Water");
    waterFolder.add(this.material.uniforms.amplitude, "value", 0, 20);
    waterFolder.add(this, "freq", 0.01, 0.1).onChange(() => {
      const d = makeDs(this.freq);
      this.material.uniforms.directions.value = d;
      this.material.uniforms.directions.value = d;
    });
    waterFolder.add(this.material.uniforms.texture_scale, "value", 100, 5000);
    waterFolder.add(this.material.uniforms.z_gain, "value", 0.1, 10);
    waterFolder
      .addColor(
        {
          scatter_color: this.material.uniforms.scatter_color.value.getHex(),
        },
        "scatter_color"
      )
      .onChange((color: string) => {
        this.material.uniforms.scatter_color.value = new THREE.Color(color);
        this.simpleMaterial.uniforms.scatter_color.value = new THREE.Color(
          color
        );
      });
    waterFolder
      .add(
        {
          scatter_factor: this.material.uniforms.scatter_factor.value,
        },
        "scatter_factor",
        1,
        500
      )
      .onChange((value: number) => {
        this.material.uniforms.scatter_factor.value = value;
        this.simpleMaterial.uniforms.scatter_factor.value = value;
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
        this.simpleMaterial.uniforms.water_color.value = new THREE.Color(color);
      });
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

  static generateHeightTexture(game: GameWasmState) {
    const textureSize = 512;
    const oceanData = game.ocean_data(textureSize);

    const texture = new THREE.DataTexture(
      oceanData,
      textureSize,
      textureSize,
      THREE.RGBAFormat
    );
    texture.magFilter = THREE.LinearFilter;
    texture.minFilter = THREE.LinearFilter;
    texture.needsUpdate = true;
    return texture;
  }

  static startWater(width: number, game: GameWasmState) {
    const heightTexture = Water.generateHeightTexture(game);
    const waterPlaneGeometry = new THREE.PlaneGeometry(
      WIDTH,
      WIDTH,
      WATER_DETAIL,
      WATER_DETAIL
    );

    Water.adjustGeometry(waterPlaneGeometry);

    const waterShader = waterCustomShader(
      heightTexture,
      game.map_size(),
      makeDs(FREQ_START),
      false
    );

    // const wireFrameMaterial = new THREE.MeshBasicMaterial({
    //   wireframe: true,
    // });

    const completeWater = new THREE.Mesh(waterPlaneGeometry, waterShader);
    // const completeWater = new THREE.Mesh(waterPlaneGeometry, wireFrameMaterial);
    completeWater.renderOrder = RenderOrder.Water;

    const simpleWaterShader = waterCustomShader(
      heightTexture,
      game.map_size(),
      makeDs(0),
      true
    );

    const simplePlane = new THREE.Mesh(
      new THREE.PlaneGeometry(width * 2, width * 2, 5, 5),
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
      new THREE.PlaneGeometry(width * 2, width * 2, 1, 1),
      new THREE.MeshBasicMaterial({
        color: 0x000000,
        visible: false,
      })
    );

    return new Water(
      waterShader,
      simpleWaterShader,
      intersectionPlane,
      watergroup
    );
  }

  private static adjustGeometry(waterPlaneGeometry: THREE.PlaneGeometry) {
    const arrPositions = waterPlaneGeometry.attributes.position.array;
    const posLength = arrPositions.length;
    const maxDistance = WIDTH / 2;

    for (let i = 0; i < posLength; i += 3) {
      const x = arrPositions[i];
      const y = arrPositions[i + 1];

      const layer = Math.max(Math.abs(x), Math.abs(y));
      const factor = (layer / maxDistance) ** 2;
      const xNormalized = x * factor;
      const yNormalized = y * factor;

      arrPositions[i] = round(xNormalized);
      arrPositions[i + 1] = round(yNormalized);
    }

    waterPlaneGeometry.attributes.position.needsUpdate = true;
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

    this.waterGroup.position.set(
      round(camera.position.x),
      round(camera.position.y),
      0
    );
  }
}

function waterCustomShader(
  heightTexture: THREE.Texture,
  mapSize: number,
  ds: THREE.Vec2[],
  stencil: boolean
) {
  const textureLoader = new THREE.TextureLoader();
  const normalTexture = textureLoader.load(normalMap);
  normalTexture.wrapS = THREE.RepeatWrapping;
  normalTexture.wrapT = THREE.RepeatWrapping;
  normalTexture.needsUpdate = true;

  return new THREE.ShaderMaterial({
    vertexShader: vertShader,
    fragmentShader: fragShader,
    blending: THREE.NormalBlending,
    transparent: false,
    depthWrite: false,
    depthTest: true,
    uniforms: {
      time: { value: 0.0 },
      directions: {
        value: ds,
      },
      map_size: { value: mapSize },
      normal_map: { value: normalTexture },
      scatter_color: { value: new THREE.Color("#f2b361") },
      water_color: { value: new THREE.Color("#10658e") },
      scatter_factor: { value: 150 },
      amplitude: { value: 2 },
      sunPosition: { value: new THREE.Vector3(1, 1, 1) },
      texture_scale: { value: 1000 },
      z_gain: { value: 1.6 },
      height_texture: { value: heightTexture },
    },
    premultipliedAlpha: false,
    stencilWrite: stencil,
    stencilFunc: THREE.EqualStencilFunc,
    stencilRef: 0,
  });
}

function makeDs(freq: number) {
  return [...Array(DIR)].map((_, i) =>
    new THREE.Vector2(Math.random(), Math.random())
      .normalize()
      .multiplyScalar((i + 1) * freq)
  );
}
