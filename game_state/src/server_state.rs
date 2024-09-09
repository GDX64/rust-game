use crate::{
    bullet::Bullet,
    diffing::Diff,
    game_map::{IslandData, WorldGrid, V2D, V3D},
    hashgrid::{HashEntity, HashEntityKind, HashGrid},
    world_gen::{self, WorldGen},
};
use cgmath::InnerSpace;
use log::info;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{borrow::BorrowMut, collections::BTreeMap, sync::Arc};
use wasm_bindgen::prelude::*;

const TOTAL_HIT: f64 = 30.0;
const BLAST_RADIUS: f64 = 20.0;
const BOAT_SPEED: f64 = 8.0;
const EXPLOSION_TTL: f64 = 1.0;
const CANON_RELOAD_TIME: f64 = 5.0;
const SHIP_SIZE: f64 = 10.0;
const SHIP_PRODUCTION_TIME: f64 = 10.0;
const ISLAND_TAKE_TIME: f64 = 1.0;
const MAX_PLAYER_SHIPS: usize = 120;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GameConstants {
    pub wind_speed: (f64, f64, f64),
    pub err_per_m: f64,
}

impl Default for GameConstants {
    fn default() -> Self {
        Self {
            wind_speed: (0.0, 0.0, 0.0),
            err_per_m: 0.01,
        }
    }
}

impl GameConstants {
    pub fn error_margin(&self, target: V2D, pos: V2D) -> Option<f64> {
        let d = target - pos;
        let err = d.magnitude() * self.err_per_m;
        return Some(err);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PlayerState {
    name: String,
    position: (f64, f64),
    id: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ShipState {
    pub position: (f64, f64),
    pub speed: (f64, f64),
    pub orientation: (f64, f64),
    pub acceleration: (f64, f64),
    pub id: u64,
    pub player_id: u64,
    pub cannon_times: [f64; 3],
    pub last_shoot_time: f64,
    pub hp: f64,
}

impl ShipState {
    pub fn key(&self) -> ShipKey {
        ShipKey::new(self.id, self.player_id)
    }

    fn to_hash_entity(&self) -> HashEntity {
        let key = self.key();
        HashEntity {
            entity: HashEntityKind::Boat(key),
            position: self.position.into(),
        }
    }
}

impl Default for ShipState {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            speed: (0.0, 0.0),
            orientation: (1.0, 0.0),
            acceleration: (0.0, 0.0),
            id: 0,
            player_id: 0,
            cannon_times: [0.0, 0.0, 0.0],
            last_shoot_time: 0.0,
            hp: 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArtifactGen {
    current_id: u64,
}

impl ArtifactGen {
    pub fn new() -> Self {
        Self { current_id: 0 }
    }

    pub fn next(&mut self) -> u64 {
        self.current_id += 1;
        self.current_id
    }
}

impl ShipState {
    pub fn find_available_cannon(&self, current_time: f64) -> Option<usize> {
        for (i, time) in self.cannon_times.iter().enumerate() {
            if current_time - time > CANON_RELOAD_TIME {
                return Some(i);
            }
        }
        None
    }

    pub fn shoot_at(&mut self, current_time: f64, target: (f64, f64)) -> Option<Bullet> {
        let cannon_index = self.find_available_cannon(current_time)?;
        let position: V2D = self.position.into();
        let ship_orientation = V2D::from(self.orientation);
        let cannon_multiplier = (cannon_index as i32 - 1) as f64 * SHIP_SIZE / 2.0;
        let cannon_pos = position + ship_orientation * cannon_multiplier;
        self.mark_shoot_time(cannon_index, current_time);

        let bullet = Bullet {
            bullet_id: 0,
            player_id: self.player_id,
            ..Bullet::maybe_from_target(cannon_pos.into(), target.into())?
        };
        self.last_shoot_time = current_time;
        return Some(bullet);
    }

    pub fn mark_shoot_time(&mut self, cannon: usize, current_time: f64) {
        self.cannon_times[cannon] = current_time;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadCastState {
    players: BTreeMap<u64, PlayerState>,
    ships: BTreeMap<ShipKey, ShipState>,
    bullets: BTreeMap<(u64, u64), Bullet>,
    explosions: BTreeMap<u64, Explosion>,
    game_constants: GameConstants,
    island_dynamic: BTreeMap<u64, IslandDynamicData>,
    artifact_gen: ArtifactGen,
    current_time: f64,
    rng_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastStateDiff {
    bullets: Vec<Diff<(u64, u64), Bullet>>,
}

impl BroadCastState {
    pub fn new() -> Self {
        Self {
            players: BTreeMap::new(),
            ships: BTreeMap::new(),
            bullets: BTreeMap::new(),
            explosions: BTreeMap::new(),
            island_dynamic: BTreeMap::new(),
            artifact_gen: ArtifactGen::new(),
            current_time: 5.0,
            rng_seed: 0,
            game_constants: GameConstants::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StateMessage {
    Shoot {
        ship_id: u64,
        player_id: u64,
        target: (f64, f64),
    },
    SetPlayerName {
        name: String,
        id: u64,
    },
    CreateShip {
        ship: ShipState,
    },
    MoveShip {
        acceleration: (f64, f64),
        id: u64,
        player_id: u64,
    },
    BroadCastState {
        state: BroadCastState,
    },
    CreatePlayer {
        id: u64,
    },
    RemovePlayer {
        id: u64,
    },
    GameConstants {
        constants: GameConstants,
    },
    Tick(f64),
    None,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ShipKey {
    pub id: u64,
    pub player_id: u64,
}

impl ShipKey {
    pub fn new(id: u64, player_id: u64) -> Self {
        Self { id, player_id }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum ExplosionKind {
    Bullet,
    Ship,
    Shot,
}

type ShipCollection = BTreeMap<ShipKey, ShipState>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Explosion {
    pub position: (f64, f64),
    pub id: u64,
    pub player_id: u64,
    pub time_created: f64,
    pub kind: ExplosionKind,
}

pub type GameMap = WorldGrid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IslandDynamicData {
    pub owner: Option<u64>,
    pub take_progress: f64,
    pub production_progress: f64,
    pub id: u64,
}

#[derive(Debug, Clone)]
pub struct ServerFlags {
    pub map_changed: bool,
}

#[derive(Clone)]
pub struct ServerState {
    pub players: BTreeMap<u64, PlayerState>,
    pub explosions: BTreeMap<u64, Explosion>,
    pub game_map: Arc<GameMap>,
    pub world_gen: Arc<world_gen::WorldGen>,
    pub bullets: BTreeMap<(u64, u64), Bullet>,
    pub island_dynamic: BTreeMap<u64, IslandDynamicData>,
    pub ship_collection: ShipCollection,
    pub current_time: f64,
    pub game_constants: GameConstants,
    pub hash_grid: HashGrid,
    rng: fastrand::Rng,
    artifact_gen: ArtifactGen,
    pub flags: ServerFlags,
}

impl ServerState {
    pub fn new() -> Self {
        let world_gen = Arc::new(world_gen::WorldGen::new(5));
        let game_map = Arc::new(world_gen.generate_grid());
        let hash_grid = HashGrid::new(game_map.dim, game_map.tile_size);
        let mut me = Self {
            game_map,
            world_gen,
            current_time: 0.0,
            artifact_gen: ArtifactGen::new(),
            explosions: BTreeMap::new(),
            players: BTreeMap::new(),
            bullets: BTreeMap::new(),
            island_dynamic: BTreeMap::new(),
            ship_collection: ShipCollection::new(),
            game_constants: GameConstants {
                wind_speed: (0.0, 0.0, 0.0),
                err_per_m: 0.01,
            },
            hash_grid,
            rng: fastrand::Rng::with_seed(0),
            flags: ServerFlags { map_changed: true },
        };
        me.fill_island_dynamic();
        return me;
    }

    pub fn minimap(&self) -> Vec<i16> {
        self.game_map
            .data
            .iter()
            .map(|x| {
                match x.island_number {
                    Some(num) => {
                        if let Some(owner) = self
                            .island_dynamic
                            .get(&num)
                            .and_then(|island| island.owner)
                        {
                            //owner id
                            return owner as i16;
                        } else {
                            //island with no owner
                            -2
                        }
                    }
                    //water
                    None => -1,
                }
            })
            .collect::<_>()
    }

    fn fill_island_dynamic(&mut self) {
        for island in self.game_map.all_island_data() {
            self.island_dynamic.insert(
                island.id,
                IslandDynamicData {
                    owner: None,
                    take_progress: 0.0,
                    production_progress: 0.0,
                    id: island.id,
                },
            );
        }
    }

    fn update_hashgrid(&mut self) {
        let mut hash_grid = HashGrid::new(self.game_map.dim, Bullet::max_distance());
        for state in self.ship_collection.values() {
            hash_grid.insert(state.to_hash_entity());
        }
        self.hash_grid = hash_grid;
    }

    pub fn all_islands(&self) -> Vec<IslandData> {
        self.game_map.all_island_data()
    }

    pub fn island_at(&self, x: f64, y: f64) -> Option<IslandData> {
        self.game_map.island_at(x, y)
    }

    pub fn next_artifact_id(&mut self) -> u64 {
        self.artifact_gen.next()
    }

    pub fn get_ship(&self, id: u64, player_id: u64) -> Option<&ShipState> {
        self.ship_collection.get(&ShipKey { id, player_id })
    }

    fn handle_set_player_name(&mut self, name: String, id: u64) {
        if let Some(player) = self.players.get_mut(&id) {
            player.name = name;
        }
    }

    pub fn state_message(&self) -> StateMessage {
        StateMessage::BroadCastState {
            state: self.get_broadcast_state(),
        }
    }

    pub fn get_broadcast_state(&self) -> BroadCastState {
        BroadCastState {
            players: self.players.clone(),
            ships: self.ship_collection.clone(),
            bullets: self.bullets.clone(),
            explosions: self.explosions.clone(),
            artifact_gen: self.artifact_gen.clone(),
            current_time: self.current_time,
            rng_seed: self.rng.get_seed(),
            game_constants: self.game_constants.clone(),
            island_dynamic: self.island_dynamic.clone(),
        }
    }

    fn is_ship_near_lighthouse(&self, ship: &ShipState, island_data: &[IslandData]) -> Option<u64> {
        let tile_size = self.game_map.tile_size;
        for island in island_data.iter() {
            let lt_pos: V2D = island.light_house.into();
            let ship_pos: V2D = ship.position.into();
            let distance = (lt_pos - ship_pos).magnitude();
            if distance < tile_size * 2.0 {
                return Some(island.id);
            }
        }
        return None;
    }

    fn tick(&mut self, dt: f64) {
        self.update_hashgrid();

        self.current_time += dt;
        let mut explosions = vec![];

        self.explosions.retain(|_key, explosion| {
            if self.current_time - explosion.time_created > EXPLOSION_TTL {
                return false;
            }
            return true;
        });

        let artifact_gen = self.artifact_gen.borrow_mut();

        self.bullets.retain(|_key, bullet| {
            bullet.evolve(dt);

            if !bullet.is_finished() {
                return true;
            };

            let pos: V3D = bullet.target.into();

            for (_id, ship) in self.ship_collection.iter_mut() {
                let ship_pos: V3D = (ship.position.0, ship.position.1, 0.0).into();
                let distance = (ship_pos - pos).magnitude();
                ship.hp -= calc_damage(distance);
            }

            let explosion = Explosion {
                position: (pos.x, pos.y),
                id: artifact_gen.next(),
                time_created: self.current_time,
                player_id: bullet.player_id,
                kind: ExplosionKind::Bullet,
            };

            explosions.push(explosion);

            return false;
        });

        self.ship_collection.retain(|_id, ship| {
            let position: V2D = ship.position.into();
            let speed: V2D = ship.speed.into();
            let acc: V2D = ship.acceleration.into();
            let speed = speed + acc * dt;
            let speed = if speed.magnitude() > BOAT_SPEED {
                speed.normalize() * BOAT_SPEED
            } else {
                speed
            };
            if speed.magnitude() > 0.001 {
                ship.orientation = speed.normalize().into();
            }
            let position = position + speed * dt;

            ship.position = position.into();
            ship.speed = speed.into();

            if ship.hp > 0.0 {
                return true;
            }

            let explosion = Explosion {
                position: ship.position,
                id: self.artifact_gen.next(),
                time_created: self.current_time,
                player_id: ship.player_id,
                kind: ExplosionKind::Ship,
            };

            explosions.push(explosion);

            return false;
        });

        for explosion in explosions {
            self.explosions.insert(explosion.id, explosion);
        }

        self.tick_handle_island_takes(dt);
        self.tick_handle_ship_production(dt);
    }

    fn tick_handle_island_takes(&mut self, dt: f64) {
        let progress = dt / ISLAND_TAKE_TIME;
        let all_island_data = self.game_map.all_island_data();
        let all_island_data = all_island_data.as_slice();
        for ship in self.ship_collection.values() {
            if let Some(island) = self.is_ship_near_lighthouse(ship, all_island_data) {
                if let Some(island) = self.island_dynamic.get_mut(&island) {
                    if island.owner != Some(ship.player_id) {
                        island.take_progress -= progress;
                        if island.take_progress <= 0.0 {
                            island.owner = Some(ship.player_id);
                            self.flags.map_changed = true;
                        }
                    } else {
                        island.take_progress += progress;
                    }
                    island.take_progress = island.take_progress.min(1.0).max(0.0);
                }
            }
        }
    }

    fn tick_handle_ship_production(&mut self, dt: f64) {
        let progress_delta = dt / SHIP_PRODUCTION_TIME;
        let mut ships_to_create = vec![];
        for island in self.island_dynamic.values_mut() {
            if let Some(owner) = island.owner {
                island.production_progress += progress_delta;
                if island.production_progress > 1.0 {
                    if let Some(island_data) = self.game_map.islands.get(&island.id) {
                        island.production_progress = 0.0;
                        let mut ship = ShipState::default();
                        ship.player_id = owner;
                        ship.position = island_data.light_house.into();
                        ships_to_create.push(ship);
                    }
                }
            }
        }
        ships_to_create.into_iter().for_each(|ship| {
            self.on_message(StateMessage::CreateShip { ship: ship });
        });
    }

    pub fn clear_flags(&mut self) {
        self.flags.map_changed = false;
    }

    pub fn get_ships(&self) -> Vec<ShipState> {
        self.ship_collection.values().cloned().collect()
    }

    pub fn get_bullets(&self) -> Vec<&Bullet> {
        self.bullets.values().collect()
    }

    pub fn on_string_message(&mut self, msg: String) -> anyhow::Result<StateMessage> {
        let msg: StateMessage = serde_json::from_str(&msg)?;
        self.on_message(msg.clone());
        Ok(msg)
    }

    fn is_ship_here(&self, x: f64, y: f64) -> bool {
        self.ship_collection.values().any(|ship| {
            let pos = ship.position;
            let dx = pos.0 - x;
            let dy = pos.1 - y;
            dx * dx + dy * dy < SHIP_SIZE * SHIP_SIZE
        })
    }

    pub fn on_message(&mut self, msg: StateMessage) {
        match msg {
            StateMessage::Tick(dt) => {
                self.tick(dt);
            }
            StateMessage::SetPlayerName { name, id } => {
                self.handle_set_player_name(name, id);
            }
            StateMessage::CreatePlayer { id } => {
                self.players.insert(
                    id,
                    PlayerState {
                        name: "Player".to_string(),
                        position: (0.0, 0.0),
                        id,
                    },
                );
            }
            StateMessage::RemovePlayer { id } => {
                self.players.remove(&id);
                self.ship_collection.retain(|_, ship| ship.player_id != id);
                self.island_dynamic.iter_mut().for_each(|(_, island)| {
                    if island.owner == Some(id) {
                        island.owner = None;
                        self.flags.map_changed = true;
                    }
                });
            }
            StateMessage::BroadCastState { state } => {
                self.ship_collection = state.ships;
                self.players = state.players;
                self.bullets = state.bullets;
                self.explosions = state.explosions;
                self.artifact_gen = state.artifact_gen;
                self.current_time = state.current_time;
                self.rng.seed(state.rng_seed);
                self.game_constants = state.game_constants;
                self.island_dynamic = state.island_dynamic;
                self.flags.map_changed = true;
                info!("Broadcast state received");
            }
            StateMessage::CreateShip { mut ship } => {
                let player_ships = self
                    .ship_collection
                    .values()
                    .filter(|s| s.player_id == ship.player_id)
                    .count();
                if player_ships >= MAX_PLAYER_SHIPS {
                    return;
                }
                ship.id = self.next_artifact_id();
                if let Some(place) =
                    self.game_map
                        .spiral_search(ship.position.0, ship.position.1, |x, y, tile| {
                            if tile.can_go() {
                                return !self.is_ship_here(x, y);
                            }
                            return false;
                        })
                {
                    ship.position = place;
                    self.ship_collection
                        .insert(ShipKey::new(ship.id, ship.player_id), ship);
                }
            }
            StateMessage::MoveShip {
                id,
                acceleration,
                player_id,
                ..
            } => {
                if let Some(ship) = self.ship_collection.get_mut(&ShipKey { id, player_id }) {
                    ship.acceleration = acceleration;
                    if acceleration.0 == 0.0 && acceleration.1 == 0.0 {
                        ship.speed = (0.0, 0.0);
                    }
                }
            }
            StateMessage::Shoot {
                ship_id,
                player_id,
                target,
            } => {
                self.handle_shoot(ship_id, player_id, target);
            }
            StateMessage::GameConstants { constants } => {
                self.game_constants = constants;
            }
            StateMessage::None => {}
        }
    }

    fn handle_shoot(&mut self, ship_id: u64, player_id: u64, target: (f64, f64)) -> Option<()> {
        let ship = self
            .ship_collection
            .get_mut(&ShipKey::new(ship_id, player_id))?;
        let pos: V2D = ship.position.into();
        let target: V2D = target.into();

        let error_mod = self.game_constants.error_margin(target, pos)?;
        let error_direction: V2D = (self.rng.f64() - 0.5, self.rng.f64() - 0.5).into();
        let target = error_direction.normalize() * error_mod * self.rng.f64() + target;

        let mut bullet = ship.shoot_at(self.current_time, target.into())?;

        bullet.bullet_id = self.artifact_gen.next();

        self.bullets
            .insert((bullet.player_id, bullet.bullet_id), bullet);

        let bullet_pos = bullet.current_pos();
        let explosion = Explosion {
            position: (bullet_pos.x, bullet_pos.y),
            id: self.artifact_gen.next(),
            time_created: self.current_time,
            player_id: player_id,
            kind: ExplosionKind::Shot,
        };

        self.explosions.insert(explosion.id, explosion);
        Some(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_rng() {
        let mut rng = fastrand::Rng::with_seed(0);
        println!("RNG: {}", rng.f64());
        println!("RNG: {}", rng.f64());
        println!("RNG: {}", rng.f64());
        println!("RNG: {:?}", rng.get_seed());
    }
}

fn calc_damage(distance: f64) -> f64 {
    if distance < BLAST_RADIUS {
        let hit_factor = 1.0 - distance / BLAST_RADIUS;
        return TOTAL_HIT * hit_factor * hit_factor;
    }
    return 0.0;
}
