import { map, Subject } from "rxjs";
import { GameWasmState } from "../pkg/game_state";
import { Linscale } from "./Linscale";
import {
  IslandData,
  IslandOwners,
  PlayerInfo,
  ShipData,
} from "./RustWorldTypes";
import { flagColors, getFlagImage } from "./PlayerStuff";
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

    const mapSize = Math.floor(
      Math.min(window.innerWidth, window.innerHeight) * minimapPercentage
    );
    this.mapSizeInPixels = mapSize * devicePixelRatio;

    mapCanvas.style.width = mapSize + "px";

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
    const mapData = islandData
      .map((island) => {
        const p2d = new Path2D();
        const path: null | [number, number][] = this.game.get_island_path(
          BigInt(island.id),
          errorMargin
        );
        if (!path || path.length < 2) {
          return null;
        }
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
      })
      .filter((x): x is [number, IslandShape] => x != null);

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

    const scale = Linscale.fromPoints(0, 0, PLANE_WIDTH, this.mapSizeInPixels);
    const xOnCanvas = scale.scale(cameraPosition.x);
    const yOnCanvas = scale.scale(cameraPosition.y);

    ctx.save();
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

    type Result = {
      center: [number, number];
      count: number;
    };

    //draw boats
    const players: Map<number, PlayerInfo> = this.game.get_all_players();
    ctx.globalAlpha = 0.9;
    const MAX_RADIUS = 12;
    const MAX_RADIUS_COUNT = 100;
    ctx.strokeStyle = "#ffffff";
    [...players.values()].flatMap((players) => {
      const result: Result[] = this.game.get_all_center_of_player(players.id);
      // ctx.fillStyle = flagColors(players.flag) ?? "#ffffff";
      const texture = getFlagImage(players.flag);
      result.forEach(({ center: [x, y], count }) => {
        ctx.save();
        ctx.beginPath();
        x = scale.scale(x);
        y = scale.scale(y);
        const factor = Math.sqrt(Math.min(1, count / MAX_RADIUS_COUNT));
        const radius = MAX_RADIUS * factor;
        ctx.ellipse(x - 1, y - 1, radius, radius, 0, 0, Math.PI * 2);
        ctx.stroke();
        ctx.clip();
        const l = radius * 2;
        ctx.drawImage(texture, x - l, y - l, 2 * l, 2 * l);
        ctx.restore();
        // ctx.fillStyle = "#ffffff";
        // ctx.fill();
      });
    });
    ctx.restore();
  }
}
