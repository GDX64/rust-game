import { GameWasmState } from "../pkg/game_state";
import * as THREE from "three";

export class Terrain {
  constructor(
    private gameState: GameWasmState,
    private readonly PLANE_WIDTH: number,
    private readonly PLANE_SEGMENTS: number,
    private readonly planeMesh: THREE.Mesh
  ) {}

  addToScene(scene: THREE.Scene) {
    scene.add(this.planeMesh);
  }

  updateMesh() {
    const { geometry } = this.planeMesh;
    const arr = geometry.attributes.position.array;
    for (let x = 0; x < this.PLANE_SEGMENTS; x += 1) {
      for (let y = 0; y < this.PLANE_SEGMENTS; y += 1) {
        const i = (y * this.PLANE_SEGMENTS + x) * 3;
        const yProportion = y / this.PLANE_SEGMENTS;
        let height =
          this.gameState.get_land_value(
            (x / this.PLANE_SEGMENTS) * this.PLANE_WIDTH - this.PLANE_WIDTH / 2,
            (0.5 - yProportion) * this.PLANE_WIDTH
          ) ?? 0;
        height = height * 500;

        arr[i + 2] = height;
      }
    }
    geometry.attributes.position.needsUpdate = true;
    geometry.computeVertexNormals();
  }

  static new(gameState: GameWasmState) {
    const PLANE_WIDTH = gameState.map_size();
    const SEGMENTS_DENSITY = gameState.tile_size();
    const PLANE_SEGMENTS = PLANE_WIDTH / SEGMENTS_DENSITY;
    const planeGeometry = new THREE.PlaneGeometry(
      PLANE_WIDTH,
      PLANE_WIDTH,
      PLANE_SEGMENTS - 1,
      PLANE_SEGMENTS - 1
    );

    // scene.fog = new THREE.Fog(0x999999, 0, 100);

    const planeMaterial = new THREE.MeshLambertMaterial({
      color: 0x00ff00,
    });

    const plane = new THREE.Mesh(planeGeometry, planeMaterial);

    return new Terrain(gameState, PLANE_WIDTH, PLANE_SEGMENTS, plane);
  }
}
