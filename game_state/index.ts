import init, { GameNoise } from "./pkg/game_state.js";
import { GUI } from "dat.gui";

const randSeed = Math.floor(Math.random() * 1000000);

function defaultParams() {
  return {
    frequency: 1,
    offsetY: 0,
    offsetX: 0,
    forestFrequency: 1,
    frequency2: 1,
    weight1: 0.9,
    threshold: 0.5,
    forestsThreshold: 0.5,
    octaves: 3,
    octaves2: 3,
    seed: randSeed,
    lacunarity: 2.0,
    persistence: 0.5,
    resolution: 200,
    scale: 50,
    field: "#00ff00",
    sea: "#0000ff",
    mountain: "#ffffff",
    sand: "#ffff00",
    forest: "#00ff00",
    rgbField: [0, 255, 0],
    rgbSea: [0, 0, 255],
    rgbMountain: [255, 255, 255],
    rgbSand: [255, 255, 0],
    rgbForest: [0, 255, 0],
  };
}

const storedParams = localStorage.getItem("params");
const params: ReturnType<typeof defaultParams> = storedParams
  ? { ...defaultParams(), ...JSON.parse(storedParams) }
  : defaultParams();

init().then(() => {
  const canvas = document.createElement("canvas");
  canvas.style.width = "800px";
  canvas.style.height = "800px";
  canvasDragEvents(canvas, draw);
  document.body.appendChild(canvas);
  const ctx = canvas.getContext("2d")!;

  const gui = new GUI();
  //add gui with a callback

  const mapGen = new GameMap(params.seed);
  const { landLowFreqNoise, landHighFreqNoise, forestGen } = mapGen;

  landHighFreqNoise.set_frequency(params.frequency2);
  landLowFreqNoise.set_octaves(params.octaves);
  landLowFreqNoise.set_frequency(params.frequency);

  gui.add(params, "seed", 0, 1000000).onChange((value) => {
    landLowFreqNoise.set_seed(Math.floor(value));
    draw();
  });

  gui.add(params, "threshold", 0, 1).onChange((value) => {
    draw();
  });

  gui.add(params, "forestsThreshold", 0, 1).onChange((value) => {
    draw();
  });

  gui.add(params, "persistence", 0, 1).onChange((value) => {
    landLowFreqNoise.set_persistence(value);
    draw();
  });

  gui.addColor(params, "sea").onChange((value) => {
    params.rgbSea = hexToRgb(params.sea);
    draw();
  });
  gui.addColor(params, "forest").onChange((value) => {
    params.rgbForest = hexToRgb(params.forest);
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
    landHighFreqNoise.set_frequency(value);
    draw();
  });

  gui.add(params, "forestFrequency", 1, 1000).onChange((value) => {
    forestGen.set_frequency(value);
    draw();
  });

  gui.add(params, "octaves2", 1, 10).onChange((value) => {
    landHighFreqNoise.set_octaves(value);
    draw();
  });

  gui.add(params, "scale", 1, 1000).onChange((value) => {
    draw();
  });

  gui.add(params, "lacunarity", 0, 10).onChange((value) => {
    landLowFreqNoise.set_lacunarity(value);
    draw();
  });

  gui.add(params, "octaves", 1, 10).onChange((value) => {
    landLowFreqNoise.set_octaves(value);
    draw();
  });

  gui.add(params, "resolution", 1, 1000).onChange((value) => {
    params.resolution = Math.floor(value);
    draw();
  });

  gui.add(params, "frequency", 1, 1000).onChange((value) => {
    landLowFreqNoise.set_frequency(value);
    draw();
  });

  draw();

  function draw() {
    localStorage.setItem("params", JSON.stringify(params));
    const data = new Uint8ClampedArray(
      params.resolution * params.resolution * 4
    );
    canvas.width = params.resolution;
    canvas.height = params.resolution;
    for (let x = 0; x < params.resolution; x++) {
      for (let y = 0; y < params.resolution; y++) {
        const [r, g, b] = mapGen.calcTileColor(x, y);
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

enum TerrainKind {
  Sea,
  Field,
  Mountain,
  Sand,
  Forest,
}

function sigmoid(t: number, threshold = 0.5) {
  return 1 / (1 + Math.exp(-(t - threshold) * 100));
}

class GameMap {
  landLowFreqNoise: GameNoise;
  landHighFreqNoise: GameNoise;
  forestGen: GameNoise;
  constructor(seed: number) {
    this.landLowFreqNoise = GameNoise.new(seed);
    this.landHighFreqNoise = GameNoise.new(seed);
    this.forestGen = GameNoise.new(seed);
  }

  terrainColor(kind: TerrainKind) {
    switch (kind) {
      case TerrainKind.Sea:
        return params.rgbSea;
      case TerrainKind.Field:
        return params.rgbField;
      case TerrainKind.Mountain:
        return params.rgbMountain;
      case TerrainKind.Sand:
        return params.rgbSand;
      case TerrainKind.Forest:
        return params.rgbForest;
    }
  }

  calcTileColor(x: number, y: number) {
    const tile = this.calcTile(x, y);
    return this.terrainColor(tile);
  }

  calcTile(_x: number, _y: number) {
    const x = (_x + params.offsetX) / params.resolution / params.scale;
    const y = (_y + params.offsetY) / params.resolution / params.scale;
    const lowNoise = this.landLowFreqNoise.get(x, y);
    const highNoise = this.landHighFreqNoise.get(x, y);
    const val = params.weight1 * lowNoise + (1 - params.weight1) * highNoise;

    const terrain = this.mapTerrain(val + params.threshold);

    if (terrain === TerrainKind.Field) {
      const forest = this.forestGen.get(x, y);
      if (forest + 0.5 > params.forestsThreshold) {
        return TerrainKind.Forest;
      }
    }
    return terrain;
  }

  mapTerrain(t: number) {
    if (t > 0.72) return TerrainKind.Field;
    if (t > 0.7) return TerrainKind.Sand;
    return TerrainKind.Sea;
  }
}

function canvasDragEvents(canvas: HTMLCanvasElement, draw: () => void) {
  let point: { x: number; y: number } | null = null;
  canvas.onpointerdown = (e) => {
    point = { x: e.clientX, y: e.clientY };
  };
  canvas.onpointermove = (e) => {
    if (point) {
      const deltaX = e.clientX - point.x;
      const deltaY = e.clientY - point.y;
      console.log("move", params.offsetX);
      params.offsetY += deltaY;
      params.offsetX += deltaX;
      point = { x: e.clientX, y: e.clientY };
      draw();
    }
  };
  canvas.onpointerup = (e) => {
    point = null;
  };
}
