import {
  WorldGen,
  WorldGenConfig,
  NoiseConfig,
  ViewInfo,
  TileKind,
} from "../pkg/game_state.js";
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
    seed: randSeed,
    resolution: 200,
    scale: 0,
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

type NoiseParams = ReturnType<typeof defaultParams>;

const storedParams = localStorage.getItem("params");

const params: NoiseParams = storedParams
  ? { ...defaultParams(), ...JSON.parse(storedParams) }
  : defaultParams();

export function noiseMapGen() {
  const canvas = document.createElement("canvas");
  canvas.style.position = "absolute";
  if (window.screen.width > window.screen.height) {
    canvas.style.height = "100%";
  } else {
    canvas.style.width = "100%";
  }
  canvas.style.aspectRatio = "1/1";
  document.body.appendChild(canvas);
  const ctx = canvas.getContext("2d")!;

  const gui = new GUI();

  const mapGen = new GameMap();
  mapGen.canvasDragEvents(canvas, draw);

  gui.add(params, "seed", 0, 1000000).onChange((value) => {
    draw();
  });

  gui.add(params, "threshold", -1, 1).onChange((value) => {
    draw();
  });

  gui.add(params, "forestsThreshold", -1, 1).onChange((value) => {
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

  gui.add(params, "frequency2", 1, 100).onChange((value) => {
    draw();
  });

  gui.add(params, "forestFrequency", 1, 100).onChange((value) => {
    draw();
  });

  gui.add(params, "scale", -3, 2).onChange((value) => {
    draw();
  });

  gui.add(params, "resolution", 1, 1000).onChange((value) => {
    params.resolution = Math.floor(value);
    draw();
  });

  gui.add(params, "frequency", 1, 100).onChange((value) => {
    draw();
  });

  draw();

  async function draw() {
    localStorage.setItem("params", JSON.stringify(params));
    await new Promise((resolve) => requestAnimationFrame(resolve));
    const data = new Uint8ClampedArray(
      params.resolution * params.resolution * 4
    );
    canvas.width = params.resolution;
    canvas.height = params.resolution;
    mapGen.updateMatrix();
    const tiles = mapGen.calcTiles();
    if (!tiles) return;
    for (let x = 0; x < params.resolution; x++) {
      for (let y = 0; y < params.resolution; y++) {
        const tileIndex = y * params.resolution + x;
        const index = tileIndex * 4;
        const terrain = tiles[tileIndex];
        const [r, g, b] = mapGen.terrainColor(terrain);
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
}

function hexToRgb(hex: string): [number, number, number] {
  return hex
    .replace("#", "")
    .match(/.{1,2}/g)!
    .map((x) => parseInt(x, 16)) as any;
}

class GameMap {
  world = WorldGen.new(params.seed);

  terrainColor(kind: TileKind) {
    switch (kind) {
      case TileKind.Water:
        return params.rgbSea;
      case TileKind.Grass:
        return params.rgbField;
      case TileKind.Forest:
        return params.rgbForest;
      default:
        return params.rgbMountain;
    }
  }

  updateMatrix() {
    this.world.set_config(makeConfig());
  }

  calcTiles(): TileKind[] | null {
    return this.world.get_canvas() ?? null;
  }

  canvasDragEvents(canvas: HTMLCanvasElement, draw: () => void) {
    let point: { x: number; y: number } | null = null;
    canvas.onpointerdown = (e) => {
      const scale = linearScale(0, 800, 0, params.resolution);
      const [canvasX, canvasY] = this.world.transform_point(
        scale(e.clientX),
        scale(e.clientY)
      );
      point = { x: canvasX, y: canvasY };
    };
    canvas.onpointermove = (e) => {
      if (point) {
        const scale = linearScale(0, 800, 0, params.resolution);
        const [canvasX, canvasY] = this.world.transform_point(
          scale(e.clientX),
          scale(e.clientY)
        );
        const deltaX = canvasX - point.x;
        const deltaY = canvasY - point.y;
        params.offsetY -= deltaY;
        params.offsetX -= deltaX;
        draw();
      }
    };
    canvas.onwheel = (e) => {
      if (e.deltaY > 0) {
        params.scale += 0.1;
      } else {
        params.scale -= 0.1;
      }
      draw();
    };
    canvas.onpointerup = (e) => {
      point = null;
    };
  }
}

function linearScale(x0: number, x1: number, y0: number, y1: number) {
  const alpha = (y1 - y0) / (x1 - x0);
  const beta = y0 - alpha * x0;
  return (v: number) => v * alpha + beta;
}

function makeConfig() {
  const worldConfig = WorldGenConfig.new();

  const forestConfig = NoiseConfig.new();
  forestConfig.frequency = params.forestFrequency;
  forestConfig.seed = params.seed;

  const lowLandConfig = NoiseConfig.new();
  lowLandConfig.seed = params.seed;
  lowLandConfig.frequency = params.frequency;

  const highLandConfig = NoiseConfig.new();
  highLandConfig.frequency = params.frequency2;
  lowLandConfig.seed = params.seed;

  const viewInfo = ViewInfo.new();
  viewInfo.x_center = params.offsetX;
  viewInfo.y_center = params.offsetY;
  viewInfo.range = 10 ** Math.min(2, params.scale);
  viewInfo.pixels = params.resolution;

  worldConfig.high_land = highLandConfig;
  worldConfig.forest = forestConfig;
  worldConfig.low_land = lowLandConfig;
  worldConfig.view_info = viewInfo;

  worldConfig.land_threshold = params.threshold;
  worldConfig.forest_threshold = params.forestsThreshold;
  worldConfig.weight_low_land = params.weight1;
  worldConfig.tile_size = 0.1;

  return worldConfig;
}
