use crate::{
    boids::{BoidLike, BoidsTeam},
    bullet::Bullet,
    diffing::Diff,
    game_map::{Tile, WorldGrid, V2D, V3D},
    world_gen::{self, TileKind},
};
use cgmath::InnerSpace;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const BLAST_RADIUS: f64 = 20.0;
const BOAT_SPEED: f64 = 8.0;
const EXPLOSION_TTL: f64 = 1.0;
const CANON_RELOAD_TIME: f64 = 5.0;
const SHIP_SIZE: f64 = 10.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameConstants {
    pub wind_speed: (f64, f64, f64),
    pub err_per_m: f64,
}

impl GameConstants {
    pub fn error_margin(&self, target: V2D, pos: V2D) -> Option<f64> {
        let d = target - pos;
        let err = d.magnitude() * self.err_per_m;
        return Some(err);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
}

impl BoidLike for ShipState {
    fn update(&self, speed: V2D) -> Self {
        let mut ship = self.clone();
        let current_speed: V2D = ship.speed.into();
        let new_speed = current_speed + speed * BOAT_SPEED / 20.0;
        let new_speed = new_speed.normalize() * BOAT_SPEED;
        ship.speed = new_speed.into();
        ship
    }

    fn position(&self) -> V2D {
        self.position.into()
    }

    fn velocity(&self) -> V2D {
        self.speed.into()
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
            ..Bullet::from_target(cannon_pos.into(), target.into())
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
    artifact_id: u64,
    current_time: f64,
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
            artifact_id: 0,
            current_time: 5.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMessage {
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
        position: (f64, f64),
        speed: (f64, f64),
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

impl ClientMessage {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).map_err(|e| e.into())
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or("".to_string())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("Failed to deserialize")
    }
}

impl From<String> for ClientMessage {
    fn from(value: String) -> Self {
        ClientMessage::from_json(&value).unwrap_or(ClientMessage::None)
    }
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

type ShipCollection = BTreeMap<ShipKey, ShipState>;

impl Tile for (f64, TileKind) {
    fn can_go(&self) -> bool {
        self.1 == TileKind::Water
    }

    fn height(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explosion {
    pub position: (f64, f64),
    pub id: u64,
    pub player_id: u64,
    pub time_created: f64,
}

pub type GameMap = WorldGrid<(f64, TileKind)>;

pub struct ServerState {
    pub players: BTreeMap<u64, PlayerState>,
    pub explosions: BTreeMap<u64, Explosion>,
    pub game_map: GameMap,
    pub world_gen: world_gen::WorldGen,
    pub bullets: BTreeMap<(u64, u64), Bullet>,
    pub ship_collection: ShipCollection,
    pub current_time: f64,
    pub game_constants: GameConstants,
    rng: fastrand::Rng,
    artifact_id: u64,
}

impl ServerState {
    pub fn new() -> Self {
        let world_gen = world_gen::WorldGen::new(1);
        let game_map = world_gen.generate_grid(4_000.0);
        Self {
            current_time: 0.0,
            artifact_id: 0,
            world_gen,
            explosions: BTreeMap::new(),
            game_map,
            players: BTreeMap::new(),
            bullets: BTreeMap::new(),
            ship_collection: ShipCollection::new(),
            game_constants: GameConstants {
                wind_speed: (0.0, 0.0, 0.0),
                err_per_m: 0.01,
            },
            rng: fastrand::Rng::with_seed(0),
        }
    }

    pub fn next_artifact_id(&mut self) -> u64 {
        self.artifact_id += 1;
        self.artifact_id
    }

    pub fn get_ship(&self, id: u64, player_id: u64) -> Option<&ShipState> {
        self.ship_collection.get(&ShipKey { id, player_id })
    }

    fn handle_set_player_name(&mut self, name: String, id: u64) {
        if let Some(player) = self.players.get_mut(&id) {
            player.name = name;
        }
    }

    pub fn state_message(&self) -> ClientMessage {
        ClientMessage::BroadCastState {
            state: self.get_broadcast_state(),
        }
    }

    pub fn get_broadcast_state(&self) -> BroadCastState {
        BroadCastState {
            players: self.players.clone(),
            ships: self.ship_collection.clone(),
            bullets: self.bullets.clone(),
            explosions: self.explosions.clone(),
            artifact_id: self.artifact_id,
            current_time: self.current_time,
        }
    }

    fn tick(&mut self, dt: f64) {
        self.current_time += dt;
        let mut ships_hit: Vec<ShipKey> = vec![];
        let mut explosions = vec![];

        self.explosions.retain(|_key, explosion| {
            if self.current_time - explosion.time_created > EXPLOSION_TTL {
                return false;
            }
            return true;
        });

        self.bullets.retain(|_key, bullet| {
            bullet.evolve(dt);

            if !bullet.is_finished() {
                return true;
            };

            let pos: V3D = bullet.target.into();

            for (id, ship) in self.ship_collection.iter() {
                if ship.player_id == bullet.player_id {
                    continue;
                }
                let ship_pos: V3D = (ship.position.0, ship.position.1, 0.0).into();
                let distance = (ship_pos - pos).magnitude();
                if distance < BLAST_RADIUS {
                    ships_hit.push(*id);
                }
            }

            explosions.push(((pos.x, pos.y), bullet.player_id));

            return false;
        });

        explosions.into_iter().for_each(|(pos, player_id)| {
            let explosion = Explosion {
                position: pos,
                id: self.next_artifact_id(),
                time_created: self.current_time,
                player_id,
            };
            self.explosions.insert(explosion.id, explosion);
        });

        self.ship_collection.retain(|id, ship| {
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
            return !ships_hit.contains(id);
        });

        //update boid style
        let all_ships = self.players.iter().flat_map(|(_, player)| {
            let ships = self.ship_collection.values().filter(|ship| {
                return ship.player_id == player.id;
            });
            let updated = BoidsTeam::update_boids_like(ships.cloned().collect());
            return updated;
        });

        self.ship_collection = all_ships
            .map(|ship| (ShipKey::new(ship.id, ship.player_id), ship))
            .collect();
    }

    pub fn get_ships(&self) -> Vec<ShipState> {
        self.ship_collection.values().cloned().collect()
    }

    pub fn get_bullets(&self) -> Vec<&Bullet> {
        self.bullets.values().collect()
    }

    pub fn on_string_message(&mut self, msg: String) -> anyhow::Result<ClientMessage> {
        let msg: ClientMessage = serde_json::from_str(&msg)?;
        self.on_message(msg.clone());
        Ok(msg)
    }

    pub fn on_message(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::Tick(dt) => {
                self.tick(dt);
            }
            ClientMessage::SetPlayerName { name, id } => {
                self.handle_set_player_name(name, id);
            }
            ClientMessage::CreatePlayer { id } => {
                self.players.insert(
                    id,
                    PlayerState {
                        name: "Player".to_string(),
                        position: (0.0, 0.0),
                        id,
                    },
                );
            }
            ClientMessage::RemovePlayer { id } => {
                self.players.remove(&id);
                self.ship_collection.retain(|_, ship| ship.player_id != id);
            }
            ClientMessage::BroadCastState { state } => {
                self.ship_collection = state.ships;
                self.players = state.players;
                self.bullets = state.bullets;
                self.explosions = state.explosions;
                self.artifact_id = state.artifact_id;
                self.current_time = state.current_time;
            }
            ClientMessage::CreateShip { mut ship } => {
                ship.id = self.next_artifact_id();
                self.ship_collection
                    .insert(ShipKey::new(ship.id, ship.player_id), ship);
            }
            ClientMessage::MoveShip {
                id,
                acceleration,
                player_id,
                ..
            } => {
                if let Some(ship) = self.ship_collection.get_mut(&ShipKey { id, player_id }) {
                    ship.acceleration = acceleration;
                }
            }
            ClientMessage::Shoot {
                ship_id,
                player_id,
                target,
            } => {
                self.handle_shoot(ship_id, player_id, target);
            }
            ClientMessage::GameConstants { constants } => {
                self.game_constants = constants;
            }
            ClientMessage::None => {}
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
        bullet.bullet_id = self.next_artifact_id();
        self.bullets
            .insert((bullet.player_id, bullet.bullet_id), bullet);
        Some(())
    }
}
