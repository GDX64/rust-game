import { GLTFLoader } from "three/examples/jsm/Addons.js";
import { GameWasmState } from "../pkg/game_state";
import { playerColor } from "./PlayerStuff";
import { IslandData, IslandOwners } from "./RustWorldTypes";
import brazil from "./assets/brasil.png";
import lighthouseUrl from "./assets/lighthouse.glb?url";
import * as THREE from "three";

export class IslandsManager {
  flagSprites = new Map<bigint, THREE.Sprite>();
  lightHouseGroup = new THREE.Group();
  owners: IslandOwners;
  islandData: Map<bigint, IslandData>;

  constructor(readonly game: GameWasmState, readonly scene: THREE.Scene) {
    const { spriteMap, spriteGroup, owners, islandData } = this.makeFlags();
    this.islandData = new Map(islandData.map((island) => [island.id, island]));
    this.owners = owners;
    this.flagSprites = spriteMap;
    this.scene.add(spriteGroup);
    this.loadLighthouse();
  }

  loadLighthouse() {
    const loader = new GLTFLoader();
    loader.load(lighthouseUrl, (gltf) => {
      const lighthouse = gltf.scene;
      lighthouse.rotateX(Math.PI / 2);
      lighthouse.scale.set(10, 10, 10);
      lighthouse.position.set(0, 0, 0);
      console.log(this.islandData);
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
    const owners: IslandOwners = this.game.island_owners();
    for (const [island, owner] of owners.entries()) {
      const sprite = this.flagSprites.get(island);
      if (sprite) {
        const ownerColor = playerColor(Number(owner));
        sprite.material.color.set(ownerColor);
      }
    }
  }

  makeFlags() {
    const owners: IslandOwners = this.game.island_owners();
    const islandData: IslandData[] = this.game.all_island_data();
    console.log(islandData);

    const textureLoader = new THREE.TextureLoader();
    const flagTexture = textureLoader.load(brazil);
    const material = new THREE.SpriteMaterial({
      color: 0x000000,
      map: flagTexture,
    });

    const sprites = islandData.map((island) => {
      const sprite = new THREE.Sprite(material);
      sprite.scale.set(50, 35, 1);
      sprite.position.set(island.center[0], island.center[1], 150);
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
