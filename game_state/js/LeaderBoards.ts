import { GameWasmState } from "../pkg/game_state";
import { getFlagImage } from "./PlayerStuff";
import { PlayerInfo } from "./RustWorldTypes";

const WIDTH = 300;
const HEIGHT = 300;
const UPDATE_TIME = 0.5; //seconds

export class LeaderBoards {
  time = 0;
  canvas = document.createElement("canvas");
  constructor(private game: GameWasmState) {
    this.canvas.width = WIDTH * devicePixelRatio;
    this.canvas.height = HEIGHT * devicePixelRatio;
    this.canvas.classList.add("leaderboards-canvas");
  }

  tick(dt: number) {
    this.time += dt;
    if (this.time > UPDATE_TIME) {
      this.time = 0;
      this.update();
    }
  }

  private update() {
    const ctx = this.canvas.getContext("2d")!;
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    ctx.save();
    ctx.scale(devicePixelRatio, devicePixelRatio);
    ctx.fillStyle = "#000000";
    //we need to use a monospace font
    ctx.font = "bold 14px monospace";
    const lineHeight = 20;
    const players: Map<number, PlayerInfo> = this.game.get_all_players();
    const imageWidth = 20;

    ctx.textBaseline = "top";
    ctx.save();
    ctx.translate(imageWidth, 0);
    ctx.fillText(header(), 0, 0);
    ctx.restore();

    ctx.translate(0, lineHeight);
    [...players.values()]
      .sort((a, b) => b.percentage_of_map - a.percentage_of_map)
      .forEach((p, i) => {
        ctx.save();
        ctx.translate(0, lineHeight * i);
        const flag = getFlagImage(p.flag);
        const height = Math.round((flag.height / flag.width) * imageWidth);
        ctx.drawImage(flag, 0, 0, imageWidth, height);
        ctx.translate(imageWidth, 0);
        ctx.fillText(leaderboardsFormat(p), 0, 0);
        ctx.restore();
      });
    ctx.restore();
  }
}

const SHIPS_CHARS = 5;
const NAME_CHARS = 8;
const ISLAND_PERCENT_CHARS = 5;
const ISLANDS_CHARS = 5;

function header() {
  const name = "Name".padEnd(NAME_CHARS, " ");
  const ships = "Ships".padEnd(SHIPS_CHARS, " ");
  const islandPercent = "Map %".padEnd(ISLAND_PERCENT_CHARS, " ");
  const islands = "Islands".padEnd(ISLANDS_CHARS, " ");
  return `${name} | ${ships} | ${islandPercent} | ${islands}`;
}

function leaderboardsFormat(player: PlayerInfo) {
  const name = player.name.slice(0, NAME_CHARS).padEnd(NAME_CHARS, " ");
  const ships = player.ships.toString().padEnd(SHIPS_CHARS, " ");
  const islandPercent = `${player.percentage_of_map.toFixed(1)}%`.padEnd(
    ISLAND_PERCENT_CHARS,
    " "
  );
  const islands = player.islands.toString().padEnd(ISLANDS_CHARS, " ");
  return `${name} | ${ships} | ${islandPercent} | ${islands}`;
}
