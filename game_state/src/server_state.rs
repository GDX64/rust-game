use crate::{
    bullet::Bullet,
    game_map::WorldGrid,
    hashgrid::HashGrid,
    island::IslandData,
    player_state::PlayerState,
    ship::SHIP_SIZE,
    ship::{ShipKey, ShipState},
    utils::vectors::{V2D, V3D},
    world_gen::{self},
};
use cgmath::InnerSpace;
use log::info;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{borrow::BorrowMut, collections::BTreeMap, sync::Arc};
use wasm_bindgen::prelude::*;

const TOTAL_HIT: f64 = 30.0;
const BLAST_RADIUS: f64 = 20.0;
const EXPLOSION_TTL: f64 = 1.0;
const SHIP_PRODUCTION_TIME: f64 = 10.0;
const ISLAND_TAKE_TIME: f64 = 1.0;
const MAX_PLAYER_SHIPS: usize = 100;
pub const PLAYER_START_SHIPS: usize = 20;

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
    frame: usize,
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
            frame: 0,
            game_constants: GameConstants::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StateMessage {
    Shoot {
        ship_id: u64,
        player_id: u64,
        target: V2D,
    },
    SetPlayerName {
        name: String,
        id: u64,
    },
    CreateShip {
        ship: ShipState,
    },
    MoveShip {
        speed: V2D,
        id: u64,
        player_id: u64,
    },
    BroadCastState {
        state: BroadCastState,
    },
    CreatePlayer {
        id: u64,
        name: String,
        flag: String,
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
    pub position: V2D,
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
    pub lighthouse: V2D,
    pub tiles: usize,
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
    frame: usize,
}

impl ServerState {
    pub fn new(seed: u32) -> Self {
        let world_gen = Arc::new(world_gen::WorldGen::new(seed));
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
            frame: 0,
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
                    lighthouse: island.light_house.into(),
                    tiles: island.tiles,
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
            frame: 0,
        }
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

            self.hash_grid
                .query_near((pos.x, pos.y).into(), BLAST_RADIUS)
                .filter_map(|entity| {
                    return entity.as_boat();
                })
                .for_each(|(key, _)| {
                    if let Some(ship) = self.ship_collection.get_mut(&key) {
                        let ship_pos: V3D = (ship.position.x, ship.position.y, 0.0).into();
                        let distance = (ship_pos - pos).magnitude();
                        ship.hp -= calc_damage(distance);
                        if ship.hp <= 0.0 && ship.killed_by.is_none() {
                            ship.killed_by = Some(bullet.player_id);
                        }
                    }
                });

            let explosion = Explosion {
                position: (pos.x, pos.y).into(),
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
            if speed.magnitude() > 0.001 {
                let orientation: V2D = ship.orientation.into();
                let diff = orientation - speed.normalize();
                let new_orientation = orientation - diff * dt * 5.0;
                ship.orientation = new_orientation.into();
            }
            let position = position + speed * dt;

            ship.position = position.into();
            ship.speed = speed.into();

            if self.game_map.is_forbidden_land(position.x, position.y) {
                ship.hp = 0.0;
            }

            if ship.hp > 0.0 {
                return true;
            }

            if let Some(killed_by) = ship.killed_by {
                let player = self.players.get_mut(&killed_by);
                if let Some(player) = player {
                    player.kills += 1;
                }
            }

            let ship_owner = self.players.get_mut(&ship.player_id);
            if let Some(player) = ship_owner {
                player.deaths += 1;
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
        self.tick_handle_player_stats();

        self.frame += 1;
    }

    fn tick_handle_player_stats(&mut self) {
        if self.frame % 15 != 0 {
            return;
        }
        self.players.values_mut().for_each(|player| {
            player.ships = self
                .ship_collection
                .values()
                .filter(|ship| ship.player_id == player.id)
                .count();
            let mut island_tiles = 0;
            let mut islands = 0;
            self.island_dynamic
                .values()
                .filter(|island| island.owner == Some(player.id))
                .for_each(|island| {
                    islands += 1;
                    island_tiles += island.tiles;
                });
            player.islands = islands;
            player.percentage_of_map =
                (island_tiles as f64 / self.game_map.total_island_tiles as f64) * 100.0;
        });
    }

    fn tick_handle_island_takes(&mut self, dt: f64) {
        let progress = dt / ISLAND_TAKE_TIME;
        let min_distance = self.game_map.tile_size * 2.0;
        self.island_dynamic.values_mut().for_each(|island| {
            let island_pos: V2D = island.lighthouse.into();
            self.hash_grid
                .query_near(island_pos, min_distance)
                .filter_map(|entity| {
                    return entity
                        .as_boat()
                        .and_then(|(key, _)| self.ship_collection.get(&key));
                })
                .for_each(|ship| {
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
                });
        })
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
            let dx = pos.x - x;
            let dy = pos.y - y;
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
            StateMessage::CreatePlayer { id, name, flag } => {
                self.players.insert(id, PlayerState::new(name, id, flag));
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
                log::info!("Player {} removed from the server", id);
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
                self.frame = state.frame;
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
                        .spiral_search(ship.position.x, ship.position.y, |x, y, tile| {
                            if tile.is_nav_water() {
                                return !self.is_ship_here(x, y);
                            }
                            return false;
                        })
                {
                    ship.position = place.into();
                    self.ship_collection
                        .insert(ShipKey::new(ship.id, ship.player_id), ship);
                }
            }
            StateMessage::MoveShip {
                id,
                speed,
                player_id,
                ..
            } => {
                if let Some(ship) = self.ship_collection.get_mut(&ShipKey { id, player_id }) {
                    ship.speed = speed;
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

    fn handle_shoot(&mut self, ship_id: u64, player_id: u64, target: V2D) -> Option<()> {
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
            position: bullet_pos.truncate(),
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
