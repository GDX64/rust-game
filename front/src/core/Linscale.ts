export class Linscale {
  constructor(private k: number, private b: number) {}

  static fromPoints(x1: number, y1: number, x2: number, y2: number) {
    let k = (y2 - y1) / (x2 - x1);
    let b = y1 - k * x1;
    return new Linscale(k, b);
  }

  alpha() {
    return this.k;
  }

  inverseScale() {
    return new Linscale(1 / this.k, -this.b / this.k);
  }

  scale(x: number) {
    return this.k * x + this.b;
  }
}
