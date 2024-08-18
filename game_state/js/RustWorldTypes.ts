import { ExplosionKind } from "../pkg/game_state";

export type ShipData = {
  player_id: number;
  id: number;
  position: [number, number];
  speed: [number, number];
  acceleration: [number, number];
  orientation: [number, number];
  hp: number;
};

export type Bullet = {
  position: [number, number, number];
  speed: [number, number, number];
  id: number;
  player_id: number;
};

export type ExplosionData = {
  position: [number, number];
  id: number;
  player_id: number;
  kind: ExplosionKind;
};

export type IslandData = {
  id: bigint;
  center: [number, number];
};

export type IslandOwners = Map<bigint, bigint>;
