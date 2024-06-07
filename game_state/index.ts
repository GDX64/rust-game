import init, { GameNoise } from "./pkg/game_state.js";
import { GUI } from "dat.gui";

const randSeed = Math.floor(Math.random() * 1000000);

const params = {
  frequency: 1,
  frequency2: 1,
  weight1: 0.9,
  threshould: 0.5,
  octaves: 3,
  octaves2: 3,
  seed: randSeed,
  lacunarity: 2.0,
  persistence: 0.5,
  resolution: 200,
  field: "#00ff00",
  sea: "#0000ff",
  mountain: "#ffffff",
  sand: "#ffff00",
  rgbField: [0, 255, 0],
  rgbSea: [0, 0, 255],
  rgbMountain: [255, 255, 255],
  rgbSand: [255, 255, 0],
};

init().then(() => {
  const canvas = document.createElement("canvas");
  canvas.style.width = "800px";
  canvas.style.height = "800px";
  document.body.appendChild(canvas);
  const ctx = canvas.getContext("2d")!;

  const gui = new GUI();
  const noise = GameNoise.new(randSeed);
  //add gui with a callback

  noise.set_frequency2(params.frequency2);
  noise.set_octaves(params.octaves);
  noise.set_frequency(params.frequency);

  gui.add(params, "seed", 0, 1000000).onChange((value) => {
    noise.set_seed(Math.floor(value));
    draw();
  });

  gui.add(params, "threshould", 0, 1).onChange((value) => {
    draw();
  });

  gui.add(params, "persistence", 0, 1).onChange((value) => {
    noise.set_persistence(value);
    draw();
  });

  gui.addColor(params, "sea").onChange((value) => {
    params.rgbSea = hexToRgb(params.sea);
    draw();
  });
  gui.addColor(params, "field").onChange((value) => {
    params.rgbField = hexToRgb(params.field);
    draw();
  });
  gui.addColor(params, "mountain").onChange((value) => {
    params.rgbMountain = hexToRgb(params.mountain);
    draw();
  });

  gui.addColor(params, "sand").onChange((value) => {
    params.rgbSand = hexToRgb(params.sand);
    draw();
  });

  gui.add(params, "weight1", 0, 1).onChange((value) => {
    draw();
  });

  gui.add(params, "frequency2", 1, 1000).onChange((value) => {
    noise.set_frequency2(value);
    draw();
  });

  gui.add(params, "octaves2", 1, 10).onChange((value) => {
    noise.set_octaves2(value);
    draw();
  });

  gui.add(params, "lacunarity", 0, 10).onChange((value) => {
    noise.set_lacunarity(value);
    draw();
  });

  gui.add(params, "octaves", 1, 10).onChange((value) => {
    noise.set_octaves(value);
    draw();
  });

  gui.add(params, "resolution", 1, 1000).onChange((value) => {
    params.resolution = Math.floor(value);
    draw();
  });

  gui.add(params, "frequency", 1, 1000).onChange((value) => {
    noise.set_frequency(value);
    draw();
  });

  draw();

  function draw() {
    const data = new Uint8ClampedArray(
      params.resolution * params.resolution * 4
    );
    canvas.width = params.resolution;
    canvas.height = params.resolution;
    for (let x = 0; x < params.resolution; x++) {
      for (let y = 0; y < params.resolution; y++) {
        const val =
          (noise.get(
            x / params.resolution / 50,
            y / params.resolution / 50,
            params.weight1
          ) +
            1) /
          2;
        const [r, g, b] = mapTerrain(val + params.threshould);
        const index = (y * params.resolution + x) * 4;
        data[index] = r;
        data[index + 1] = g;
        data[index + 2] = b;
        data[index + 3] = 255;
      }
    }
    ctx.putImageData(
      new ImageData(data, params.resolution, params.resolution),
      0,
      0
    );
  }
});

function hexToRgb(hex: string): [number, number, number] {
  return hex
    .replace("#", "")
    .match(/.{1,2}/g)!
    .map((x) => parseInt(x, 16)) as any;
}

function step(t: number, threshold = 0.5) {
  return t < threshold ? 0 : 1;
}

function mapTerrain(t: number) {
  if (t > 0.9) return params.rgbMountain;
  if (t > 0.72) return params.rgbField;
  if (t > 0.7) return params.rgbSand;
  return params.rgbSea;
}

function sigmoid(t: number, threshold = 0.5) {
  return 1 / (1 + Math.exp(-(t - threshold) * 100));
}
