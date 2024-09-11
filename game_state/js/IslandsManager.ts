import { GLTFLoader } from "three/examples/jsm/Addons.js";
import { GameWasmState } from "../pkg/game_state";
import { IslandData, IslandOwners } from "./RustWorldTypes";
import lighthouseUrl from "./assets/lighthouse.glb?url";
import * as THREE from "three";

const allCountries = import.meta.glob<string>("./assets/flags/*.png", {
  query: "?url",
  import: "default",
});

// const countryOptions = Object.keys(allCountries).map((key) => {
//   return key.match(/flags\/(.*)\.png/)![1];
// });

function getFlagPromise(country: string) {
  return allCountries[`./assets/flags/${country}.png`]();
}

const flagsTextures = new Map<string, THREE.Texture>();
const flagsLoading = new Map<string, Promise<void>>();

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

const loader = new THREE.TextureLoader();
export class IslandsManager {
  flagSprites = new Map<number, THREE.Sprite>();
  lightHouseGroup = new THREE.Group();
  owners: IslandOwners;
  islandData: Map<number, IslandData>;
  needsUpdate = false;

  constructor(readonly game: GameWasmState, readonly scene: THREE.Scene) {
    const { spriteMap, spriteGroup, owners, islandData } = this.makeFlags();
    this.islandData = new Map(islandData.map((island) => [island.id, island]));
    this.owners = owners;
    this.flagSprites = spriteMap;
    this.scene.add(spriteGroup);
    this.loadLighthouse();
  }

  getFlagTexture(country: string) {
    const flag = getFlagTexture(country);
    if (flag) {
      return flag;
    }
    whenFlagLoaded(country)?.then(() => {
      this.needsUpdate = true;
    });
  }

  loadLighthouse() {
    const loader = new GLTFLoader();
    const headlight = new THREE.SphereGeometry(0.15, 16, 16);
    const headlightMaterial = new THREE.MeshLambertMaterial({
      color: 0xffff00,
      // emissive: 0xffff00,
      // emissiveIntensity: 10,
    });
    const headlightMesh = new THREE.Mesh(headlight, headlightMaterial);
    loader.load(lighthouseUrl, (gltf) => {
      const lighthouse = gltf.scene;
      lighthouse.rotateX(Math.PI / 2);
      lighthouse.scale.set(10, 10, 10);
      lighthouse.position.set(0, 0, 0);
      headlightMesh.position.set(0, 5.1, 0);
      lighthouse.add(headlightMesh);
      this.islandData.forEach((island) => {
        const [x, y] = island.light_house;
        const lighthouseInstance = lighthouse.clone();
        lighthouseInstance.position.set(x, y, 0);
        this.lightHouseGroup.add(lighthouseInstance);
      });
    });
    this.scene.add(this.lightHouseGroup);
  }

  tick() {
    if (!this.game.has_map_changed() && !this.needsUpdate) {
      return;
    }
    this.needsUpdate = false;
    const owners: IslandOwners = this.game.island_owners();
    for (const [island, { owner }] of owners.entries()) {
      const sprite = this.flagSprites.get(island);
      if (!sprite) {
        continue;
      }
      if (owner != null) {
        const ownerFlag = this.game.get_player_flag(BigInt(owner));
        const flag = this.getFlagTexture(ownerFlag);
        sprite.material.map = flag ?? null;
        sprite.material.needsUpdate = true;
      } else {
        sprite.material.map = null;
      }
    }
  }

  makeFlags() {
    const owners: IslandOwners = this.game.island_owners();
    const islandData: IslandData[] = this.game.all_island_data();

    const sprites = islandData.map((island) => {
      const material = new THREE.SpriteMaterial({
        color: 0xffffff,
      });
      const sprite = new THREE.Sprite(material);
      sprite.scale.set(50, 35, 1);
      sprite.position.set(island.light_house[0], island.light_house[1], 100);
      return { sprite, island: island.id };
    });
    const spriteGroup = new THREE.Group();
    spriteGroup.add(...sprites.map(({ sprite }) => sprite));
    const spriteMap = new Map(
      sprites.map(({ island, sprite }) => [island, sprite])
    );
    return { spriteMap, spriteGroup, owners, islandData };
  }
}
