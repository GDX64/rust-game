import * as THREE from "three";
import { extractColors } from "extract-colors";

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

const allCountries = import.meta.glob<string>("./assets/flags/*.png", {
  query: "?url",
  import: "default",
});

const loader = new THREE.TextureLoader();

function getFlagPromise(country: string) {
  return allCountries[`./assets/flags/${country}.png`]();
}

const flagsTextures = new Map<string, THREE.Texture>();
const flagImages = new Map<string, HTMLImageElement>();
const flagsLoading = new Map<string, Promise<void>>();
const vibrantColors = new Map<string, string>();
const loadingVibrant = new Set<string>();

export function flagColors(country: string) {
  const cached = vibrantColors.get(country);
  if (cached) {
    return cached;
  }
  if (loadingVibrant.has(country)) {
    return null;
  }
  loadingVibrant.add(country);
  getFlagPromise(country)
    .then((url) => {
      return extractColors(url);
    })
    .then((pallet) => {
      const color = pallet[0].hex;
      if (color) {
        vibrantColors.set(country, color);
      }
    })
    .finally(() => {
      loadingVibrant.delete(country);
    });
  return null;
}

export function getFlagImage(country: string): HTMLImageElement {
  if (flagImages.has(country)) {
    return flagImages.get(country)!;
  }
  const img = new Image();
  flagImages.set(country, img);
  getFlagPromise(country).then((url) => {
    img.width = 100;
    img.src = url;
  });
  return img;
}

export function getFlagTexture(country: string) {
  if (flagsTextures.has(country)) {
    return flagsTextures.get(country);
  }
  if (flagsLoading.has(country)) {
    return null;
  }
  const loading = getFlagPromise(country)
    .then((url) => {
      const texture = loader.load(url);
      flagsTextures.set(country, texture);
    })
    .finally(() => {
      flagsLoading.delete(country);
    });
  flagsLoading.set(country, loading);
}

export function whenFlagLoaded(country: string) {
  return flagsLoading.get(country);
}
