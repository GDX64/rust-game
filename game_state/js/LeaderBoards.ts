import { GameWasmState } from "../pkg/game_state";
import { getFlagImage } from "./PlayerStuff";
import { PlayerInfo } from "./RustWorldTypes";

const PADDING = 5;
const HEIGHT = 300;
const UPDATE_TIME = 0.2; //seconds
const imageWidth = 20;

export class LeaderBoards {
  time = 0;
  width = 200;
  //we need to use a monospace font
  font = "bold 14px monospace";
  canvas = document.createElement("canvas");
  constructor(private game: GameWasmState) {
    this.resizeCanvas();
  }

  private resizeCanvas() {
    this.width = Math.ceil(this.measureHeader() + PADDING * 2 + imageWidth);
    console.log("leaderboards width", this.width);
    this.canvas.width = this.width * devicePixelRatio;
    this.canvas.height = HEIGHT * devicePixelRatio;
    this.canvas.classList.add("leaderboards-canvas");
    this.canvas.style.width = this.width + "px";
    this.canvas.style.height = HEIGHT + "px";
    this.canvas.style.transition = "opacity 0.3s";
  }

  measureHeader() {
    const ctx = this.canvas.getContext("2d")!;
    ctx.font = this.font;
    return ctx.measureText(header()).width;
  }

  showLeaderBoards() {
    this.canvas.style.opacity = "1";
  }

  hideLeaderBoards() {
    this.canvas.style.opacity = "0.3";
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
    ctx.font = this.font;
    const lineHeight = 20;
    const players: Map<number, PlayerInfo> = this.game.get_all_players();
    const imagePadding = 4;

    const playersArr = [...players.values()];
    ctx.fillStyle = "#00000088";
    const totalHeight = (playersArr.length + 1) * lineHeight + 2 * PADDING;
    ctx.fillRect(0, 0, this.width, totalHeight);
    ctx.translate(PADDING, PADDING);

    ctx.textBaseline = "top";
    ctx.fillStyle = "#ffffff";
    ctx.save();
    ctx.translate(imageWidth + imagePadding, 0);
    ctx.fillText(header(), 0, 0);
    ctx.restore();

    ctx.translate(0, lineHeight);

    playersArr
      .sort((a, b) => b.percentage_of_map - a.percentage_of_map)
      .forEach((p, i) => {
        ctx.save();
        ctx.translate(0, lineHeight * i);
        const flag = getFlagImage(p.flag);
        const height = Math.round((flag.height / flag.width) * imageWidth);
        ctx.drawImage(flag, 0, 0, imageWidth, height);
        ctx.translate(imageWidth + imagePadding, 0);
        ctx.fillText(leaderboardsFormat(p), 0, 0);
        ctx.restore();
      });
    ctx.restore();
  }
}

const SHIPS_CHARS = 4;
const NAME_CHARS = 8;
const ISLAND_PERCENT_CHARS = 5;
// const ISLANDS_CHARS = 3;
const KILLS_CHARS = 4;
// const DEATHS_CHARS = 4;

function header() {
  const name = "Name".padEnd(NAME_CHARS, " ");
  const ships = "S".padEnd(SHIPS_CHARS, " ");
  const islandPercent = "%".padEnd(ISLAND_PERCENT_CHARS, " ");
  // const islands = "LHs".padEnd(ISLANDS_CHARS, " ");
  const kills = "K".padEnd(KILLS_CHARS, " ");
  // const deaths = "D".padEnd(DEATHS_CHARS, " ");
  return `${name} | ${ships} | ${kills} | ${islandPercent}`;
}

function leaderboardsFormat(player: PlayerInfo) {
  const name = player.name.slice(0, NAME_CHARS).padEnd(NAME_CHARS, " ");
  const ships = player.ships.toString().padEnd(SHIPS_CHARS, " ");
  const islandPercent = `${Math.round(player.percentage_of_map)}%`.padEnd(
    ISLAND_PERCENT_CHARS,
    " "
  );
  // const islands = player.islands.toString().padEnd(ISLANDS_CHARS, " ");
  const kills = player.kills.toString().padEnd(KILLS_CHARS, " ");
  // const deaths = player.deaths.toString().padEnd(DEATHS_CHARS, " ");
  return `${name} | ${ships} | ${kills} | ${islandPercent}`;
}
