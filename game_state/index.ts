import init, { GameNoise } from "./pkg/game_state.js";
import { GUI } from "dat.gui";

init().then(() => {
  const canvas = document.createElement("canvas");
  document.body.appendChild(canvas);
  const ctx = canvas.getContext("2d")!;
  const SIZE = 800;
  canvas.width = SIZE;
  canvas.height = SIZE;
  const randSeed = Math.floor(Math.random() * 1000000);
  const gui = new GUI();
  const noise = GameNoise.new(randSeed);
  //add gui with a callback
  const params = {
    frequency: 0.01,
    threshould: 0.5,
    octaves: 3,
    seed: randSeed,
    lacunarity: 2.0,
    persistence: 0.5,
  };
  const data = new Uint8ClampedArray(SIZE * SIZE * 4);

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

  gui.add(params, "lacunarity", 0, 10).onChange((value) => {
    noise.set_lacunarity(value);
    draw();
  });

  gui.add(params, "octaves", 1, 10).onChange((value) => {
    noise.set_octaves(value);
    draw();
  });

  noise.set_octaves(params.octaves);
  noise.set_frequency(params.frequency);
  gui.add(params, "frequency", 0.005, 0.05).onChange((value) => {
    noise.set_frequency(value);
    draw();
  });

  draw();

  function draw() {
    for (let x = 0; x < SIZE; x++) {
      for (let y = 0; y < SIZE; y++) {
        const val = (noise.get(x, y) + 1) / 2;
        const color = step(val, params.threshould) * 255;
        const index = (y * SIZE + x) * 4;
        data[index] = color;
        data[index + 1] = color;
        data[index + 2] = color;
        data[index + 3] = 255;
      }
    }
    ctx.putImageData(new ImageData(data, SIZE, SIZE), 0, 0);
  }
});

function step(t: number, threshold = 0.5) {
  return t < threshold ? 0 : 1;
}

function sigmoid(t: number, threshold = 0.5) {
  return 1 / (1 + Math.exp(-(t - threshold) * 100));
}
