import { ExplosionKind } from "../pkg/game_state";

export type V2D = { x: number; y: number };
export type V3D = { x: number; y: number; z: number };

export type ShipData = {
  player_id: number;
  id: number;
  position: V2D;
  speed: V2D;
  acceleration: V2D;
  orientation: V2D;
  hp: number;
};

export type ShipPosByPlayer = Float64Array;

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

export type PlayerInfo = {
  id: number;
  name: string;
  flag: string;
  percentage_of_map: number;
  kills: number;
  ships: number;
  islands: number;
};
