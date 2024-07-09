import * as THREE from "three";

export class Water {
  constructor() {}

  static startWater(WIDTH: number) {
    const waterPlaneGeometry = new THREE.PlaneGeometry(WIDTH, WIDTH, 1, 1);

    const waterMaterial = new THREE.MeshPhongMaterial({
      color: 0x0000ff,
      transparent: true,
      opacity: 0.9,
      shininess: 30,
      side: THREE.DoubleSide,
      fog: true,
    });

    const waterMesh = new THREE.Mesh(waterPlaneGeometry, waterMaterial);

    return { waterPlaneGeometry, waterMaterial, waterMesh };
  }
}
