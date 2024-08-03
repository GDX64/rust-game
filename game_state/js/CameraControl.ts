import * as THREE from "three";

type V3 = THREE.Vector3;
class LerpBox {
  private time = performance.now() / 1000;
  private _duration = 1;
  constructor(
    public from: V3 = new THREE.Vector3(),
    public to: V3 = new THREE.Vector3()
  ) {}

  duration(d: number) {
    this._duration = d;
    return this;
  }

  updateTo(v: V3) {
    this.from = this.evolve();
    this.to = v;
    this.time = performance.now() / 1000;
  }

  evolve() {
    const time = performance.now() / 1000;
    let t = Math.min(1, (time - this.time) / this._duration);
    t = Math.sin((t * Math.PI) / 2);
    const now = this.from.clone().lerp(this.to, t);
    return now;
  }
}

export class CameraControl {
  private target = new LerpBox().duration(0.166);
  private position = new LerpBox().duration(0.166);
  private keys: Record<string, boolean> = {};
  private time = 0;
  constructor(public camera: THREE.Camera) {
    camera.position.z = 100;
    camera.position.y = -200;
    camera.up.set(0, 0, 1);
    camera.lookAt(this.target.to);
    this.position.updateTo(this.camera.position.clone());
  }

  private lookDirectionProjected() {
    const look = this.camera.getWorldDirection(new THREE.Vector3(0, 0, 0));
    return look.projectOnPlane(new THREE.Vector3(0, 0, 1));
  }

  tick(time: number) {
    this.handlePressedKeys();
    const target = this.target.evolve();
    const position = this.position.evolve();
    this.camera.lookAt(target);
    this.camera.position.set(position.x, position.y, position.z);
    this.time = time;
  }

  changeTarget(v: V3) {
    this.target.updateTo(v);
  }

  changePosition(v: V3) {
    this.position.updateTo(v);
  }

  addListeners() {
    document.addEventListener("keyup", (event: KeyboardEvent) => {
      this.keys[event.key] = false;
    });
    document.addEventListener("keydown", (event: KeyboardEvent) => {
      this.keys[event.key] = true;
    });
    document.addEventListener("wheel", (event: WheelEvent) => {
      this.onWeel(event);
    });
  }

  private onWeel(event: WheelEvent) {
    const delta = event.deltaY;
    const amount = delta / 500;
    const position = this.camera.position.clone();
    const target = this.target.to.clone();
    //rotate position around target
    const direction = position.clone().sub(target);
    console.log(amount);
    direction.multiplyScalar(amount);
    position.add(direction);
    this.changePosition(position);
  }

  rotateAroundZ(sign: number) {
    const amount = 0.05 * sign;
    const position = this.camera.position.clone();
    const target = this.target.to.clone();
    //rotate position around target
    target.sub(position);
    target.applyAxisAngle(new THREE.Vector3(0, 0, 1), amount);
    target.add(position);
    this.changeTarget(target);
  }

  rotateAroundPlane(sign: number) {
    const projected = this.lookDirectionProjected();
    const orthogonal = new THREE.Vector3(0, 0, 1).cross(projected);

    const amount = 0.02 * sign;
    const position = this.camera.position.clone();
    const target = this.target.to.clone();
    //rotate position around target
    target.sub(position);
    target.applyAxisAngle(orthogonal, amount);
    target.add(position);
    this.changeTarget(target);
  }

  private handlePressedKeys() {
    if (this.keys.q || this.keys.e) {
      const amount = this.keys.q ? 1 : -1;
      this.rotateAroundZ(amount);
      return;
    }
    if (this.keys.W || this.keys.S) {
      const sign = this.keys.W ? 1 : -1;
      const up = new THREE.Vector3(0, 0, 1);
      const delta = up.normalize().multiplyScalar(sign * 10);
      this.changeTarget(this.target.to.clone().add(delta));
      this.changePosition(this.position.to.clone().add(delta));
      return;
    }
    if (this.keys["w"] || this.keys["s"]) {
      const sign = this.keys.w ? 1 : -1;
      const projected = this.lookDirectionProjected();
      const delta = projected.normalize().multiplyScalar(sign * 10);
      this.changeTarget(this.target.to.clone().add(delta));
      this.changePosition(this.position.to.clone().add(delta));
      return;
    }
    if (this.keys["a"] || this.keys["d"]) {
      const sign = this.keys.a ? 1 : -1;
      const projected = this.lookDirectionProjected();
      const up = new THREE.Vector3(0, 0, 1);
      const right = up.cross(projected);
      const delta = right.normalize().multiplyScalar(sign * 10);
      this.changeTarget(this.target.to.clone().add(delta));
      this.changePosition(this.position.to.clone().add(delta));
      return;
    }
  }
}
