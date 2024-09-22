import { ExplosionKind } from "../pkg/game_state";

type V2D = { x: number; y: number };
type V3D = { x: number; y: number; z: number };

export type ShipData = {
  player_id: number;
  id: number;
  position: V2D;
  speed: V2D;
  acceleration: V2D;
  orientation: V2D;
  hp: number;
};

export type Bullet = {
  position: V3D;
  speed: V3D;
  id: number;
  player_id: number;
};

export type ExplosionData = {
  position: V2D;
  id: number;
  player_id: number;
  kind: ExplosionKind;
};

export type PlayerState = {
  id: number;
  name: string;
};

export type IslandData = {
  id: number;
  center: [number, number];
  light_house: [number, number];
};

export type IslandOwners = Map<
  number,
  {
    take_progress: number;
    owner?: number;
  }
>;
