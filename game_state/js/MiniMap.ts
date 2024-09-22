import { Subject } from "rxjs";
import { GameWasmState } from "../pkg/game_state";
import { Linscale } from "./Linscale";
import { IslandData, IslandOwners } from "./RustWorldTypes";
import { getFlagImage } from "./PlayerStuff";
import * as THREE from "three";

const minimapPercentage = 0.25;

type IslandShape = {
  path: Path2D;
  x: number;
  y: number;
  width: number;
  height: number;
  id: number;
};

export class MiniMap {
  islandShapes: Map<number, IslandShape>;
  islandsCanvas;
  mapCanvas;
  mapClick$ = new Subject<{ x: number; y: number }>();
  needUpdate = false;

  mapSizeInPixels;
  constructor(private game: GameWasmState) {
    const mapCanvas = document.createElement("canvas");
    mapCanvas.classList.add("minimap-canvas");

    this.mapSizeInPixels =
      Math.min(window.innerWidth, window.innerHeight) *
      minimapPercentage *
      devicePixelRatio;

    mapCanvas.width = this.mapSizeInPixels;
    mapCanvas.height = this.mapSizeInPixels;
    const PLANE_WIDTH = this.game.map_size();

    mapCanvas.onclick = (event: MouseEvent) => {
      const { height, width } = mapCanvas.getBoundingClientRect();
      const miniMapToWorldX = Linscale.fromPoints(
        0,
        -PLANE_WIDTH / 2,
        width,
        PLANE_WIDTH / 2
      );
      const miniMapToWorldY = Linscale.fromPoints(
        0,
        PLANE_WIDTH / 2,
        height,
        -PLANE_WIDTH / 2
      );
      event.stopPropagation();
      const x = event.offsetX;
      const y = event.offsetY;
      const xWorld = miniMapToWorldX.scale(x);
      const yWorld = miniMapToWorldY.scale(y);
      this.mapClick$.next({ x: xWorld, y: yWorld });
    };

    const islandsCanvas = new OffscreenCanvas(
      this.mapSizeInPixels,
      this.mapSizeInPixels
    );

    this.islandsCanvas = islandsCanvas;
    this.mapCanvas = mapCanvas;

    this.islandShapes = this.buildShapes();
  }

  private buildShapes() {
    const islandData: IslandData[] = this.game.all_island_data();
    const ctx = this.islandsCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);

    const mapSize = this.game.map_size();
    const scaleX = Linscale.fromPoints(
      -mapSize / 2,
      0,
      mapSize / 2,
      this.mapSizeInPixels
    );
    const scaleY = Linscale.fromPoints(
      mapSize / 2,
      0,
      -mapSize / 2,
      this.mapSizeInPixels
    );

    const errorMargin = scaleX.inverseScale().alpha() * 10;
    const mapData = islandData.map((island) => {
      const p2d = new Path2D();
      const path: [number, number][] = this.game.get_island_path(
        BigInt(island.id),
        errorMargin
      );
      const x = scaleX.scale(path[0][0]);
      const y = scaleY.scale(path[0][1]);
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
        id: island.id,
      };
      return [island.id, shape] as const;
    });

    return new Map(mapData);
  }

  private updateIslands() {
    const ctx = this.islandsCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
    const owners: IslandOwners = this.game.island_owners();
    this.islandShapes.forEach((shape) => {
      const owner = owners.get(shape.id)?.owner;
      ctx.save();

      ctx.lineWidth = 2;
      ctx.strokeStyle = "#000000";
      ctx.stroke(shape.path);

      if (owner != null) {
        const country = this.game.get_player_flag(BigInt(owner));
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
        ctx.fillStyle = "#616161";
        ctx.fill(shape.path);
      }
      ctx.restore();
    });
  }

  updateCanvas(camera: THREE.Camera) {
    if (this.game.has_map_changed() || this.needUpdate) {
      this.updateIslands();
      this.needUpdate = false;
    }

    const ctx = this.mapCanvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.mapSizeInPixels, this.mapSizeInPixels);
    ctx.drawImage(this.islandsCanvas, 0, 0);

    const cameraPosition = camera.position;
    const cameraDirection = camera.getWorldDirection(new THREE.Vector3());

    //draw triangle for camera
    ctx.fillStyle = "#ffff00";
    const rotationOnXY = Math.atan2(cameraDirection.y, cameraDirection.x);

    ctx.save();

    ctx.scale(1, -1);
    ctx.translate(this.mapSizeInPixels / 2, -this.mapSizeInPixels / 2);
    const PLANE_WIDTH = this.game.map_size();

    const xOnCanvas = (cameraPosition.x / PLANE_WIDTH) * this.mapSizeInPixels;
    const yOnCanvas = (cameraPosition.y / PLANE_WIDTH) * this.mapSizeInPixels;

    ctx.translate(xOnCanvas, yOnCanvas);
    ctx.rotate(rotationOnXY);

    const ARROW_SIZE = 9;

    const arrowSize = ARROW_SIZE * devicePixelRatio;
    ctx.beginPath();
    ctx.moveTo(0, -arrowSize / 3);
    ctx.lineTo(arrowSize, 0);
    ctx.lineTo(0, arrowSize / 3);
    ctx.fill();
    ctx.restore();
  }
}
