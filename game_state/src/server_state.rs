use cgmath::InnerSpace;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
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
    pub id: u64,
    pub player_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadCastState {
    players: Vec<PlayerState>,
    ships: Vec<ShipState>,
    bullets: Vec<Bullet>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bullet {
    pub position: (f64, f64),
    pub speed: (f64, f64),
    pub player_id: u64,
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

impl ClientMessage {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).map_err(|e| e.into())
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl From<String> for ClientMessage {
    fn from(value: String) -> Self {
        ClientMessage::from_json(&value).unwrap_or(ClientMessage::None)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct ShipKey {
    id: u64,
    player_id: u64,
}

impl ShipKey {
    fn new(id: u64, player_id: u64) -> Self {
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
    pub bullets: Vec<Bullet>,
    ship_collection: ShipCollection,
}

impl ServerState {
    pub fn new() -> Self {
        let world_gen = world_gen::WorldGen::new(1);
        let game_map = world_gen.generate_grid(100.0);
        Self {
            world_gen,
            game_map,
            players: HashMap::new(),
            bullets: vec![],
            ship_collection: ShipCollection::new(),
        }
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
            state: BroadCastState {
                players: self.players.values().cloned().collect(),
                ships: self.ship_collection.values().cloned().collect(),
                bullets: self.bullets.clone(),
            },
        }
    }

    pub fn tick(&mut self, dt: f64) {
        for ship in self.ship_collection.values_mut() {
            let (x, y) = ship.position;
            let (vx, vy) = ship.speed;
            let (x, y) = (x + vx * dt, y + vy * dt);
            ship.position = (x, y);
        }
        self.bullets.retain_mut(|bullet| {
            let (x, y) = bullet.position;
            let (vx, vy) = bullet.speed;
            let (x, y) = (x + vx * dt, y + vy * dt);
            bullet.position = (x, y);
            let target: V2D = bullet.target.into();
            let pos: V2D = bullet.position.into();
            let distance = (target - pos).magnitude();
            if distance < 1.0 {
                return false;
            }
            return true;
        });
    }

    pub fn get_ships(&self) -> Vec<ShipState> {
        self.ship_collection.values().cloned().collect()
    }

    pub fn on_string_message(&mut self, msg: String) -> anyhow::Result<ClientMessage> {
        let msg: ClientMessage = serde_json::from_str(&msg)?;
        self.on_message(msg.clone());
        Ok(msg)
    }

    pub fn on_message(&mut self, msg: ClientMessage) {
        info!("Received message on state: {:?}", msg);
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
                self.ship_collection = state
                    .ships
                    .into_iter()
                    .map(|ship| {
                        return (ShipKey::new(ship.id, ship.player_id), ship);
                    })
                    .collect();

                self.players = state
                    .players
                    .into_iter()
                    .map(|player| (player.id, player))
                    .collect();

                self.bullets = state.bullets;
            }
            ClientMessage::CreateShip { ship } => {
                self.ship_collection.insert(
                    ShipKey {
                        id: ship.id,
                        player_id: ship.player_id,
                    },
                    ship,
                );
            }
            ClientMessage::MoveShip {
                position,
                speed,
                id,
                player_id,
            } => {
                if let Some(ship) = self.ship_collection.get_mut(&ShipKey { id, player_id }) {
                    ship.position = position;
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
                if let Some(ship) = ship {
                    let (x, y) = ship.position;
                    let dx = target.0 - x;
                    let dy = target.1 - y;
                    let len = (dx * dx + dy * dy).sqrt();
                    let speed = 10.0;
                    let speed = (dx / len * speed, dy / len * speed);
                    let bullet = Bullet {
                        position: ship.position,
                        speed,
                        player_id,
                        target,
                    };
                    self.bullets.push(bullet);
                }
            }
            ClientMessage::None => {}
        }
    }
}
