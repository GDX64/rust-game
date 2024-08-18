import { GameWasmState } from "../pkg/game_state";
import { playerColor } from "./PlayerStuff";
import { IslandData, IslandOwners } from "./RustWorldTypes";
import brazil from "./assets/brasil.png";

import * as THREE from "three";

export class IslandsManager {
  flagSprites = new Map<bigint, THREE.Sprite>();
  owners: IslandOwners;

  constructor(readonly game: GameWasmState, readonly scene: THREE.Scene) {
    const { spriteMap, spriteGroup, owners } = this.makeFlags();
    this.owners = owners;
    this.flagSprites = spriteMap;
    this.scene.add(spriteGroup);
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
    return { spriteMap, spriteGroup, owners };
  }
}
