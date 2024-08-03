import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

export class CameraControl {
  orbit: OrbitControls;
  constructor(camera: THREE.Camera, element: HTMLElement) {
    camera.position.z = 100;
    camera.position.y = -200;
    camera.up.set(0, 0, 1);
    this.orbit = new OrbitControls(camera, element);
    this.orbit.target.set(0, 0, 0);
    this.orbit.enableDamping = true;
  }

  get camera() {
    return this.orbit.object;
  }

  private lookDirectionProjected() {
    const look = this.camera.getWorldDirection(new THREE.Vector3(0, 0, 0));
    return look.projectOnPlane(new THREE.Vector3(0, 0, 1));
  }

  tick(_time: number) {
    this.orbit.update();
  }

  addListeners() {
    document.addEventListener("keydown", (event: KeyboardEvent) => {
      if (event.key === "w" || event.key === "s") {
        const sign = event.key === "w" ? 1 : -1;
        const projected = this.lookDirectionProjected();
        const delta = projected.normalize().multiplyScalar(sign * 10);
        this.orbit.target.add(delta);
        this.camera.position.add(delta);
      }
      if (event.key === "a" || event.key === "d") {
        const sign = event.key === "a" ? 1 : -1;
        const projected = this.lookDirectionProjected();
        const up = new THREE.Vector3(0, 0, 1);
        const right = up.cross(projected);
        const delta = right.normalize().multiplyScalar(sign * 10);
        this.camera.position.add(delta);
        this.orbit.target.add(delta);
      }
      this.orbit.update();
    });
  }
}
