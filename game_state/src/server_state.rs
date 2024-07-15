use cgmath::InnerSpace;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    diffing::{hashmap_diff, Diff},
    sparse_matrix::{CanGo, WorldGrid, V2D},
    world_gen::{self, TileKind},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerState {
    name: String,
    position: (f64, f64),
    id: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ShipState {
    pub position: (f64, f64),
    pub speed: (f64, f64),
    pub acceleration: (f64, f64),
    pub id: u64,
    pub player_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadCastState {
    players: HashMap<u64, PlayerState>,
    ships: HashMap<ShipKey, ShipState>,
    bullets: HashMap<(u64, u64), Bullet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastStateDiff {
    bullets: Vec<Diff<(u64, u64), Bullet>>,
}

impl BroadCastState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            ships: HashMap::new(),
            bullets: HashMap::new(),
        }
    }

    pub fn diff(&self, other: &Self) -> BroadcastStateDiff {
        let bullets_diff = hashmap_diff(&self.bullets, &other.bullets);
        BroadcastStateDiff {
            bullets: bullets_diff,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Bullet {
    pub position: (f64, f64),
    pub speed: (f64, f64),
    pub player_id: u64,
    pub bullet_id: u64,
    pub target: (f64, f64),
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
    None,
}

struct Line2D {
    start: V2D,
    end: V2D,
}

impl Line2D {
    fn new(start: V2D, end: V2D) -> Self {
        Self { start, end }
    }

    fn distance_to_point(&self, point: &V2D) -> f64 {
        let l2 = (self.end - self.start).magnitude();
        if l2 == 0.0 {
            return (point - self.start).magnitude();
        }
        let t = ((point - self.start).dot(self.end - self.start) / l2)
            .max(0.0)
            .min(1.0);
        let projection = self.start + (self.end - self.start) * t;
        (point - projection).magnitude()
    }
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipKey {
    pub id: u64,
    pub player_id: u64,
}

impl ShipKey {
    pub fn new(id: u64, player_id: u64) -> Self {
        Self { id, player_id }
    }
}

type ShipCollection = HashMap<ShipKey, ShipState>;

impl CanGo for (f64, TileKind) {
    fn can_go(&self) -> bool {
        self.1 == TileKind::Water
    }
}

pub type GameMap = WorldGrid<(f64, TileKind)>;

pub struct ServerState {
    pub players: HashMap<u64, PlayerState>,
    pub game_map: GameMap,
    pub world_gen: world_gen::WorldGen,
    pub bullets: HashMap<(u64, u64), Bullet>,
    pub ship_collection: ShipCollection,
    artifact_id: u64,
}

impl ServerState {
    pub fn new() -> Self {
        let world_gen = world_gen::WorldGen::new(1);
        let game_map = world_gen.generate_grid(100.0);
        Self {
            artifact_id: 0,
            world_gen,
            game_map,
            players: HashMap::new(),
            bullets: HashMap::new(),
            ship_collection: ShipCollection::new(),
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
        }
    }

    pub fn tick(&mut self, dt: f64) {
        let mut ships_hit: Vec<ShipKey> = vec![];

        self.bullets.retain(|_key, bullet| {
            let (x, y) = bullet.position;
            let (vx, vy) = bullet.speed;
            let (x, y) = (x + vx * dt, y + vy * dt);
            let bullet_initial: V2D = bullet.position.into();
            let bullet_final: V2D = (x, y).into();
            let line = Line2D::new(bullet_initial, bullet_final);

            for (id, ship) in self.ship_collection.iter() {
                if ship.player_id == bullet.player_id {
                    continue;
                }
                let ship_position: V2D = ship.position.into();
                let distance = line.distance_to_point(&ship_position);
                if distance < 1.0 {
                    info!("hit ship: {:?}", id);
                    ships_hit.push(*id);
                    return false;
                }
            }

            bullet.position = (x, y);
            let target: V2D = bullet.target.into();
            let pos: V2D = bullet.position.into();
            let distance = (target - pos).magnitude();

            return distance > 1.0;
        });

        self.ship_collection.retain(|id, ship| {
            let position: V2D = ship.position.into();
            let speed: V2D = ship.speed.into();
            let acc: V2D = ship.acceleration.into();
            let speed = speed + acc * dt;
            let speed = if speed.magnitude() > 0.5 {
                speed.normalize() / 2.0
            } else {
                speed
            };
            let position = position + speed * dt;
            ship.position = position.into();
            ship.speed = speed.into();
            return !ships_hit.contains(id);
        });
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
            }
            ClientMessage::CreateShip { ship } => {
                self.ship_collection
                    .insert(ShipKey::new(ship.id, ship.player_id), ship);
            }
            ClientMessage::MoveShip {
                position,
                speed,
                id,
                acceleration,
                player_id,
            } => {
                if let Some(ship) = self.ship_collection.get_mut(&ShipKey { id, player_id }) {
                    ship.position = position;
                    ship.acceleration = acceleration;
                    ship.speed = speed;
                }
            }
            ClientMessage::Shoot {
                ship_id,
                player_id,
                target,
            } => {
                let ship = self.ship_collection.get(&ShipKey {
                    id: ship_id,
                    player_id,
                });
                let (speed, position) = if let Some(ship) = ship {
                    let (x, y) = ship.position;
                    let dx = target.0 - x;
                    let dy = target.1 - y;
                    let len = (dx * dx + dy * dy).sqrt();
                    let speed = 10.0;
                    let speed = (dx / len * speed, dy / len * speed);
                    (speed, ship.position)
                } else {
                    return;
                };
                let bullet = Bullet {
                    bullet_id: self.next_artifact_id(),
                    position,
                    speed,
                    player_id,
                    target,
                };
                self.bullets
                    .insert((bullet.player_id, bullet.bullet_id), bullet);
            }
            ClientMessage::None => {}
        }
    }
}
