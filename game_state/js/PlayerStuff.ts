import * as THREE from "three";

const P1 = new THREE.Color("#1b69cf");
const P2 = new THREE.Color("#e43131");
const P3 = new THREE.Color("#35d435");
const P4 = new THREE.Color("#d8d840");
const P5 = new THREE.Color("#d643d6");
const P6 = new THREE.Color("#43d8d8");
const P7 = new THREE.Color("#ff8800");
const P8 = new THREE.Color("#f83e9b");
const P9 = new THREE.Color("#3f6fb8");
const playerArray = [P1, P2, P3, P4, P5, P6, P7, P8, P9];

export function playerColor(playerID: number) {
  return playerArray[playerID % playerArray.length];
}
