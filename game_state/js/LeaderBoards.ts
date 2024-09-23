import { GameWasmState } from "../pkg/game_state";
import { getFlagImage } from "./PlayerStuff";
import { PlayerInfo } from "./RustWorldTypes";

const SIZE = 0.25;
const UPDATE_TIME = 0.5; //seconds

export class LeaderBoards {
  time = 0;
  canvas = document.createElement("canvas");
  constructor(private game: GameWasmState) {
    this.canvas.width = Math.floor(window.innerWidth * SIZE * devicePixelRatio);
    this.canvas.height = Math.floor(
      window.innerHeight * SIZE * devicePixelRatio
    );
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
    ctx.fillStyle = "black";
    //we need to use a monospace font
    ctx.font = "12px monospace";
    const lineHeight = 20;
    const players: Map<number, PlayerInfo> = this.game.get_all_players();
    players.forEach((p, i) => {
      ctx.save();
      ctx.translate(0, lineHeight * i);
      const flag = getFlagImage(p.flag);
      const imageWidth = 20;
      const height = Math.round((flag.height / flag.width) * imageWidth);
      ctx.drawImage(flag, 0, 0, imageWidth, height);
      ctx.translate(imageWidth, 0);
      ctx.textBaseline = "top";
      ctx.fillText(`${p.name} - ${p.ships}`, 0, 0);
      ctx.restore();
    });
    ctx.restore();
  }
}
