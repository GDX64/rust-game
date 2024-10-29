import { Subject } from "rxjs";
import { GameWasmState } from "rust";
import { Linscale } from "./Linscale";
import {
  CenterResults,
  IslandData,
  IslandOwners,
  PlayerInfo,
} from "./RustWorldTypes";
import { getFlagImage } from "./PlayerStuff";
import * as THREE from "three";

const minimapPercentage = 0.33;

type IslandShape = {
  path: Path2D;
  x: number;
  y: number;
  width: number;
  height: number;
  id: number | null;
};

export class MiniMap {
  private islandShapes: Map<number, IslandShape>;
  private mapSizeInPixels;
  private smallIslandShapes: IslandShape[];
  private lastDrawMatrix: DOMMatrix = new DOMMatrix();
  islandsCanvas;
  mapCanvas;
  mapClick$ = new Subject<{ x: number; y: number }>();
  needUpdate = false;

  constructor(private game: GameWasmState) {
    const mapCanvas = document.createElement("canvas");
    mapCanvas.classList.add("minimap-canvas");

    const mapSize = Math.floor(
      Math.min(window.innerWidth, window.innerHeight) * minimapPercentage
    );
    this.mapSizeInPixels = mapSize * devicePixelRatio;

    mapCanvas.style.width = mapSize + "px";
    mapCanvas.style.transition = "opacity 0.3s";
    mapCanvas.style.border = "2px solid #fff1a1";
    mapCanvas.style.borderRadius = "100%";
    mapCanvas.style.right = "3px";
    mapCanvas.style.bottom = "3px";
    mapCanvas.style.position = "absolute";

    mapCanvas.width = this.mapSizeInPixels;
    mapCanvas.height = this.mapSizeInPixels;

    const islandsCanvas = new OffscreenCanvas(
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );

    this.islandsCanvas = islandsCanvas;
    this.mapCanvas = mapCanvas;

    this.islandShapes = this.buildShapes();
    this.smallIslandShapes = this.buildSmallShapes();
    this.addCanvasEvents();
  }

  private addCanvasEvents() {
    const mapCanvas = this.mapCanvas;

    let isDragging = false;

    document.onpointerup = (_event: PointerEvent) => {
      isDragging = false;
    };

    document.onpointerleave = (_event: PointerEvent) => {
      isDragging = false;
    };

    mapCanvas.onpointerdown = (event: PointerEvent) => {
      isDragging = true;
      emitMoveEvent(event);
    };

    mapCanvas.onpointermove = (event: PointerEvent) => {
      if (isDragging) {
        emitMoveEvent(event);
      }
    };

    const emitMoveEvent = (event: MouseEvent) => {
      const mapSize = this.game.map_size();
      const { height, width } = mapCanvas.getBoundingClientRect();
      const miniMapToWorldX = Linscale.fromPoints(
        0,
        -mapSize / 2,
        width,
        mapSize / 2
      );
      const miniMapToWorldY = Linscale.fromPoints(
        0,
        mapSize / 2,
        height,
        -mapSize / 2
      );

      event.stopPropagation();
      const x = event.offsetX;
      const y = event.offsetY;
      const point = new DOMPoint(x, y);
      const transformedPoint = point.matrixTransform(
        this.lastDrawMatrix.inverse()
      );

      const xWorld = miniMapToWorldX.scale(transformedPoint.x);
      const yWorld = miniMapToWorldY.scale(transformedPoint.y);
      this.mapClick$.next({ x: xWorld, y: yWorld });
    };
  }

  private scalePair() {
    const mapSize = this.game.map_size();
    const pixelPadding = 4;
    const scaleX = Linscale.fromPoints(
      -mapSize / 2,
      pixelPadding,
      mapSize / 2,
      this.mapSizeInPixels - pixelPadding
    );
    const scaleY = Linscale.fromPoints(
      mapSize / 2,
      pixelPadding,
      -mapSize / 2,
      this.mapSizeInPixels - pixelPadding
    );
    return { scaleX, scaleY };
  }

  private buildSmallShapes() {
    const { scaleX } = this.scalePair();
    const errorMargin = scaleX.inverseScale().alpha() * 10;
    const islandData: [number, number][][] =
      this.game.get_small_island_paths(errorMargin);
    const mapData = islandData
      .map((path) => {
        if (path.length < 2) {
          return null;
        }
        const shape = this.makeIslandPath(path);
        return shape;
      })
      .filter((x) => x != null);
    return mapData;
  }

  private buildShapes() {
    const islandData: IslandData[] = this.game.all_island_data();

    const { scaleX } = this.scalePair();
    const errorMargin = scaleX.inverseScale().alpha() * 10;
    const mapData = islandData
      .map((island) => {
        const path: null | [number, number][] = this.game.get_island_path(
          BigInt(island.id),
          errorMargin
        );
        if (!path || path.length < 2) {
          return null;
        }
        const shape = this.makeIslandPath(path);
        shape.id = island.id;
        return [island.id, shape];
      })
      .filter((x): x is [number, IslandShape] => x != null);

    return new Map(mapData);
  }

  private makeIslandPath(path: [number, number][]) {
    const { scaleX, scaleY } = this.scalePair();
    const x = scaleX.scale(path[0][0]);
    const y = scaleY.scale(path[0][1]);
    const p2d = new Path2D();
    p2d.moveTo(x, y);
    let minX = x;
    let minY = y;
    let maxX = x;
    let maxY = y;
    path.slice(1).forEach(([x, y]) => {
      const scaledX = scaleX.scale(x);
      const scaledY = scaleY.scale(y);
      p2d.lineTo(scaledX, scaledY);
      minX = Math.min(minX, scaledX);
      minY = Math.min(minY, scaledY);
      maxX = Math.max(maxX, scaledX);
      maxY = Math.max(maxY, scaledY);
    });

    const islandWidth = maxX - minX;
    const islandHeight = maxY - minY;

    p2d.closePath();
    const shape: IslandShape = {
      path: p2d,
      width: islandWidth,
      height: islandHeight,
      x: minX,
      y: minY,
      id: null,
    };
    return shape;
  }

  private updateIslands() {
    const ctx = this.islandsCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
    const owners: IslandOwners = this.game.island_owners();
    const islands = [...this.islandShapes.values(), ...this.smallIslandShapes];
    islands.forEach((shape) => {
      const isSmallIsland = shape.id == null;
      const owner = shape.id != null ? owners.get(shape.id)?.owner : null;
      ctx.save();

      if (isSmallIsland) {
        ctx.globalAlpha = 0.7;
      } else {
        ctx.lineWidth = 2;
        ctx.strokeStyle = "#000000";
        ctx.stroke(shape.path);
      }

      if (owner != null) {
        const country = this.game.get_player_flag(BigInt(owner));
        if (!country) {
          return;
        }
        const img = getFlagImage(country);
        if (img.width) {
          ctx.clip(shape.path);
          const drawHeight = Math.ceil(Math.max(shape.width, shape.height));
          const aspectRatio = img.width / img.height;
          ctx.drawImage(
            img,
            shape.x,
            shape.y,
            Math.floor(drawHeight * aspectRatio),
            drawHeight
          );
        } else {
          img.onload = () => {
            this.needUpdate = true;
          };
        }
      } else {
        ctx.fillStyle = "#7c7c7c";
        ctx.fill(shape.path);
      }

      ctx.restore();
    });
  }

  hideMinimap() {
    this.mapCanvas.style.opacity = "0.3";
    this.mapCanvas.style.pointerEvents = "none";
  }

  showMinimap() {
    this.mapCanvas.style.opacity = "1";
    this.mapCanvas.style.pointerEvents = "auto";
  }

  private currentMatrix(camera: THREE.Camera) {
    const cameraDirection = camera.getWorldDirection(new THREE.Vector3());
    const rotationOnXY = Math.atan2(cameraDirection.y, cameraDirection.x);
    const rotationOnXYYFromY = rotationOnXY + (3 * Math.PI) / 2;
    const matrix = new DOMMatrix();
    matrix.translateSelf(this.mapSizeInPixels / 2, this.mapSizeInPixels / 2);
    const rotationDegrees = (rotationOnXYYFromY * 180) / Math.PI;
    matrix.rotateSelf(0, 0, rotationDegrees);
    matrix.translateSelf(-this.mapSizeInPixels / 2, -this.mapSizeInPixels / 2);
    return { matrix, rotationOnXY, rotationOnXYYFromY };
  }

  updateCanvas(camera: THREE.Camera) {
    if (this.game.has_map_changed() || this.needUpdate) {
      this.updateIslands();
      this.needUpdate = false;
    }

    const ctx = this.mapCanvas.getContext("2d")!;
    const { matrix, rotationOnXY, rotationOnXYYFromY } =
      this.currentMatrix(camera);

    this.lastDrawMatrix = matrix;

    const cameraPosition = camera.position;

    ctx.save();
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
    ctx.setTransform(matrix);
    ctx.drawImage(this.islandsCanvas, 0, 0);

    const PLANE_WIDTH = this.game.map_size();

    const scaleX = Linscale.fromPoints(
      -PLANE_WIDTH / 2,
      0,
      PLANE_WIDTH / 2,
      this.mapSizeInPixels
    );
    const scaleY = Linscale.fromPoints(
      PLANE_WIDTH / 2,
      0,
      -PLANE_WIDTH / 2,
      this.mapSizeInPixels
    );
    const xOnCanvas = scaleX.scale(cameraPosition.x);
    const yOnCanvas = scaleY.scale(cameraPosition.y);

    ctx.save();
    //draw triangle for camera
    ctx.fillStyle = "#ffff00";
    ctx.translate(xOnCanvas, yOnCanvas);

    ctx.rotate(-rotationOnXY);

    const ARROW_SIZE = 9;

    const arrowSize = ARROW_SIZE * devicePixelRatio;
    ctx.beginPath();
    ctx.moveTo(0, -arrowSize / 3);
    ctx.lineTo(arrowSize, 0);
    ctx.lineTo(0, arrowSize / 3);
    ctx.fill();
    ctx.restore();

    //draw boats
    const players: Map<number, PlayerInfo> = this.game.get_all_players();
    ctx.globalAlpha = 0.9;
    const MAX_RADIUS = 12;
    const MAX_RADIUS_COUNT = 100;
    const flagAspectRatio = 0.67;
    ctx.strokeStyle = "#ffffff";
    for (const playerInfo of players.values()) {
      const result: CenterResults[] = this.game.get_all_center_of_player(
        playerInfo.id
      );
      // ctx.fillStyle = flagColors(players.flag) ?? "#ffffff";
      const texture = getFlagImage(playerInfo.flag);
      result.forEach(({ center: [x, y], count }) => {
        ctx.save();
        const factor = Math.sqrt(Math.min(1, count / MAX_RADIUS_COUNT));
        const radius = Math.floor(MAX_RADIUS * factor);
        x = scaleX.scale(x) - radius;
        y = scaleY.scale(y) - radius;
        x = Math.floor(x);
        y = Math.floor(y);
        const width = radius * 2;
        const height = Math.floor(width * flagAspectRatio);
        ctx.beginPath();
        ctx.translate(x, y);
        ctx.rotate(-rotationOnXYYFromY);
        ctx.rect(0, 0, width, height);
        ctx.stroke();
        ctx.clip();

        ctx.drawImage(texture, 0, 0, width, height);
        ctx.restore();
      });
    }

    //Draw NORTH
    ctx.globalAlpha = 1;
    const NORTH_MARGIN = 15;
    const yNorth = scaleY.scale(PLANE_WIDTH / 2) + NORTH_MARGIN;
    const xNorth = scaleX.scale(0);
    ctx.fillStyle = "#fff1a1";
    ctx.font = "16px sans-serif";
    ctx.fillText("N", xNorth, yNorth);

    ctx.restore();
  }
}
