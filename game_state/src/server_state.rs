use crate::{
    diffing::{hashmap_diff, Diff},
    game_map::{Tile, WorldGrid, V2D, V3D},
    world_gen::{self, TileKind},
    Boids::{BoidLike, BoidsTeam},
};
use cgmath::{InnerSpace, MetricSpace};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BULLET_SPEED: f64 = 200.0;
const GRAVITY: f64 = 9.81;
const BLAST_RADIUS: f64 = 20.0;
const BOAT_SPEED: f64 = 8.0;
const EXPLOSION_TTL: f64 = 1.0;
const CANON_RELOAD_TIME: f64 = 5.0;
const SHIP_SIZE: f64 = 10.0;
const WIND_FACTOR: f64 = 0.01;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameConstants {
    pub wind_speed: (f64, f64, f64),
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
    players: HashMap<u64, PlayerState>,
    ships: HashMap<ShipKey, ShipState>,
    bullets: HashMap<(u64, u64), Bullet>,
    explosions: HashMap<u64, Explosion>,
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
            players: HashMap::new(),
            ships: HashMap::new(),
            bullets: HashMap::new(),
            explosions: HashMap::new(),
            artifact_id: 0,
            current_time: 5.0,
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
    pub position: (f64, f64, f64),
    pub speed: (f64, f64, f64),
    pub player_id: u64,
    pub bullet_id: u64,
    pub target: (f64, f64, f64),
}

impl Bullet {
    pub fn from_target(initial: V2D, target: V2D) -> Bullet {
        let v0 = BULLET_SPEED;
        let g = GRAVITY;
        let initial: V3D = (initial.x, initial.y, 0.0).into();
        let target: V3D = (target.x, target.y, 0.0).into();
        let d_vector = target - initial;
        let d = d_vector.magnitude();
        let angle = f64::asin(d * g / (2.0 * v0 * v0));
        let angle = if angle.is_nan() { 3.14 / 4.0 } else { angle };
        let vxy = v0 * f64::cos(angle);
        let vz = v0 * f64::sin(angle);
        let vx = d_vector.normalize() * vxy;
        let speed = (vx.x, vx.y, vz).into();
        Bullet {
            position: initial.into(),
            speed,
            player_id: 0,
            bullet_id: 0,
            target: target.into(),
        }
    }

    pub fn error_margin(&self, game_constants: &GameConstants) -> Option<f64> {
        let mut clone = self.clone();
        for _ in 0..1000 {
            clone.evolve(0.016, game_constants);
            if clone.position.2 <= 0.0 {
                let final_pos: V3D = (clone.position.0, clone.position.1, 0.0).into();
                let target: V3D = clone.target.into();
                return Some(final_pos.distance(target));
            }
        }
        return None;
    }

    pub fn evolve(&mut self, dt: f64, game_constants: &GameConstants) {
        let speed: V3D = self.speed.into();
        let pos: V3D = self.position.into();
        let speed = speed + dt * V3D::new(0.0, 0.0, -GRAVITY);
        let pos = pos + speed * dt;
        let wind_diff = V3D::from(game_constants.wind_speed) - speed;
        let speed = speed + wind_diff * WIND_FACTOR * dt;
        self.position = pos.into();
        self.speed = speed.into();
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
    None,
}

// mod line2D {
//     struct Line2D {
//         start: V2D,
//         end: V2D,
//     }

//     impl Line2D {
//         fn new(start: V2D, end: V2D) -> Self {
//             Self { start, end }
//         }

//         fn distance_to_point(&self, point: &V2D) -> f64 {
//             let l2 = (self.end - self.start).magnitude();
//             if l2 == 0.0 {
//                 return (point - self.start).magnitude();
//             }
//             let t = ((point - self.start).dot(self.end - self.start) / l2)
//                 .max(0.0)
//                 .min(1.0);
//             let projection = self.start + (self.end - self.start) * t;
//             (point - projection).magnitude()
//         }
//     }
// }

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
    pub players: HashMap<u64, PlayerState>,
    pub explosions: HashMap<u64, Explosion>,
    pub game_map: GameMap,
    pub world_gen: world_gen::WorldGen,
    pub bullets: HashMap<(u64, u64), Bullet>,
    pub ship_collection: ShipCollection,
    pub current_time: f64,
    pub game_constants: GameConstants,
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
            explosions: HashMap::new(),
            game_map,
            players: HashMap::new(),
            bullets: HashMap::new(),
            ship_collection: ShipCollection::new(),
            game_constants: GameConstants {
                wind_speed: (0.0, 0.0, 0.0),
            },
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

    pub fn tick(&mut self, dt: f64) {
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
            bullet.evolve(dt, &self.game_constants);
            let pos: V3D = bullet.position.into();

            if pos.z > self.game_map.height_of(pos.x, pos.y).max(0.0) {
                return true;
            };

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
                info!("Creating ship: {:?}", ship);
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
                let bullet = self
                    .ship_collection
                    .get_mut(&ShipKey::new(ship_id, player_id))
                    .and_then(|ship| ship.shoot_at(self.current_time, target));
                if let Some(mut bullet) = bullet {
                    bullet.bullet_id = self.next_artifact_id();
                    self.bullets
                        .insert((bullet.player_id, bullet.bullet_id), bullet);
                }
            }
            ClientMessage::GameConstants { constants } => {
                self.game_constants = constants;
            }
            ClientMessage::None => {}
        }
    }
}

#[cfg(test)]
mod test {
    use cgmath::InnerSpace;

    use crate::{game_map::V3D, server_state::BLAST_RADIUS, Bullet};

    fn verify_hits_target(initial: (f64, f64), target: (f64, f64)) -> bool {
        let mut bullet = Bullet::from_target(initial.into(), target.into());
        for _ in 0..100 {
            bullet.evolve(
                0.016,
                &crate::server_state::GameConstants {
                    wind_speed: (0.0, 0.0, 0.0),
                },
            );
            let pos: V3D = bullet.position.into();
            let target = V3D::from(bullet.target);
            if (pos - target).magnitude() < BLAST_RADIUS {
                return true;
            }
        }
        return false;
    }

    #[test]
    fn test_shoot() {
        assert!(verify_hits_target((0.0, 0.0), (5.0, 5.0)));
        assert!(verify_hits_target((0.0, 0.0), (-5.0, 0.0)));
        assert!(verify_hits_target((0.0, 0.0), (0.0, 3.0)));
    }

    #[test]
    fn test_error_margin() {
        let bullet = Bullet::from_target((0.0, 0.0).into(), (20.0, 20.0).into());
        let error = bullet.error_margin(&crate::server_state::GameConstants {
            wind_speed: (0.0, 0.0, 0.0),
        });
        println!("Error: {:?}", error);
        // assert!(error.unwrap() < 1.0);
    }
}
