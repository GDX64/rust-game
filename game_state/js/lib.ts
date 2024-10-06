import { Render3D } from "./render3d";
import * as Lib from "./libInterface";

export class ArchpelagusGame implements Lib.ArchpelagusGame {
  private constructor(
    private renderer: Render3D,
    private element: HTMLElement
  ) {}

  static async new(element: HTMLElement) {
    const state = Render3D.state();
    const game = await Render3D.startServer(state.online);
    const renderer = Render3D.new(element, game);

    const controller = new ArchpelagusGame(renderer, element);

    let timer: any;
    window.addEventListener("resize", () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        controller.onResize();
      }, 1000);
    });
    return controller;
  }

  private onResize() {
    this.renderer.destroy();
    this.renderer = Render3D.new(this.element, this.renderer.gameState);
  }

  destroy() {
    this.renderer.destroy();
  }
}
