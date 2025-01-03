use crate::{
    bullet::Bullet,
    hashgrid::HashEntityKind,
    server_state::{ServerState, StateMessage},
    ship::{ShipKey, ShipState},
    utils::{spiral_search::SpiralSearch, vectors::V2D},
};
use anyhow::Context;
use cgmath::InnerSpace;
use log::error;
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

const BOAT_SPEED: f64 = 16.0;

#[derive(Debug)]
pub struct PlayerShip {
    path: Vec<V2D>,
    id: u64,
    should_remove: bool,
    target: Option<V2D>,
    speed: Option<V2D>,
}

pub struct Player {
    pub id: u64,
    moving_ships: HashMap<u64, PlayerShip>,
    pub selected_ships: Vec<u64>,
    actions: Sender<StateMessage>,
    actions_buffer: Receiver<StateMessage>,
    pub rng: fastrand::Rng,
    pub shoot_radius: f64,
}

impl Player {
    pub fn new(id: u64) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Player {
            id,
            moving_ships: HashMap::new(),
            actions: sender,
            actions_buffer: receiver,
            selected_ships: Vec::new(),
            rng: fastrand::Rng::with_seed(id),
            shoot_radius: 10.0,
        }
    }

    pub fn my_ships<'a>(&self, game_state: &'a ServerState) -> Vec<&'a ShipState> {
        game_state
            .ship_collection
            .values()
            .filter(|ship| ship.player_id == self.id)
            .collect()
    }

    pub fn move_selected_ships(&mut self, game_state: &ServerState, x: f64, y: f64) {
        let formation = unit_spiral_formation(self.selected_ships.len(), x, y, game_state);
        self.selected_ships
            .clone()
            .iter()
            .zip(formation)
            .for_each(|(&ship_id, (x, y))| {
                let path = self.find_path_for_ship(game_state, ship_id, x, y);
                if let Some(path) = path {
                    self.make_ship_follow_path(ship_id, path);
                }
            });
    }

    pub fn change_shoot_radius(&mut self, r: f64) {
        self.shoot_radius = r;
    }

    pub fn clear_selected_ships(&mut self) {
        self.selected_ships.clear();
    }

    pub fn selec_ship(&mut self, ship_id: u64, game: &ServerState) {
        self.selected_ships.push(ship_id);
        self.selected_ships
            .retain(|&id| game.get_ship(id, self.id).is_some())
    }

    fn make_ship_follow_path(&mut self, ship_id: u64, path: Vec<V2D>) {
        let ship = PlayerShip {
            path,
            id: ship_id,
            should_remove: false,
            target: None,
            speed: None,
        };

        self.moving_ships.insert(ship_id, ship);
    }

    pub fn find_path_for_ship(
        &mut self,
        game_state: &ServerState,
        ship_id: u64,
        x: f64,
        y: f64,
    ) -> Option<Vec<V2D>> {
        let server_ship = game_state
            .ship_collection
            .get(&ShipKey::new(ship_id, self.id))?;
        let path = game_state
            .game_map
            .find_path(server_ship.position, (x, y))?;

        //the fist one is already the current position
        let path = path[1..].to_vec();
        return Some(path);
    }

    pub fn shoot_at_with(&mut self, ship_id: u64, x: f64, y: f64) {
        let theta = self.rng.f64() * std::f64::consts::PI * 2.0;
        let r = self.rng.f64().sqrt() * self.shoot_radius;
        let error_x = r * theta.cos();
        let error_y = r * theta.sin();
        let x = x + error_x;
        let y = y + error_y;

        let msg = StateMessage::Shoot {
            ship_id,
            player_id: self.id,
            target: (x, y).into(),
        };
        if let Err(err) = self.actions.send(msg).context(file!()) {
            error!("Error sending message: {}", err)
        };
    }

    pub fn shoot_at(&mut self, target: &V2D, game_state: &ServerState) {
        let selected_number = (self.selected_ships.len() + 1) / 2;
        let ships = self
            .shooting_ships(game_state)
            .take(selected_number)
            .cloned()
            .collect::<Vec<_>>();

        for ship in ships {
            self.shoot_at_with(ship.id, target.x, target.y);
        }
    }

    pub fn select_all(&mut self, game_state: &ServerState) {
        self.selected_ships = self.player_ships(game_state).map(|ship| ship.id).collect();
    }

    pub fn select_all_idle(&mut self, game_state: &ServerState) {
        self.selected_ships = self
            .player_ships(game_state)
            .filter_map(|ship| {
                if self.moving_ships.contains_key(&ship.id) {
                    return None;
                }
                return Some(ship.id);
            })
            .collect();
    }

    pub fn auto_shoot(&mut self, game_state: &ServerState) {
        let mut shot_already = vec![];
        let mut rng = self.rng.clone();
        let pairs = self
            .shooting_ships(game_state)
            .filter_map(|ship| {
                let enemies = game_state
                    .hash_grid
                    .query_near(ship.position.into(), Bullet::max_distance())
                    .filter_map(|entity| {
                        if let HashEntityKind::Boat(key) = entity.entity {
                            if key.player_id != self.id {
                                return Some((entity.position, key));
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    });
                for (enemy_pos, key) in enemies {
                    if shot_already.contains(&key) || rng.f32() > 0.1 {
                        continue;
                    }
                    shot_already.push(key);
                    return Some((ship.id, enemy_pos));
                }
                return None;
            })
            .collect::<Vec<_>>();
        for (id, enemy) in pairs {
            self.shoot_at_with(id, enemy.x, enemy.y);
        }
        self.rng = rng;
    }

    fn shooting_ships<'a>(&'a self, game: &'a ServerState) -> impl Iterator<Item = &'a ShipState> {
        self.player_ships(game).filter(|ship| {
            let is_selected = self.selected_ships.contains(&ship.id);
            let can_shoot = ship.find_available_cannon(game.current_time).is_some();
            return is_selected && can_shoot;
        })
    }

    pub fn can_shoot_here(&self, target: V2D, game: &ServerState) -> bool {
        let mut ships = self.shooting_ships(game).filter_map(|ship| {
            Bullet::maybe_from_target(ship.position.into(), target)?;
            return Some(());
        });
        return ships.next().is_some();
    }

    pub fn player_ships<'a>(&self, game: &'a ServerState) -> impl Iterator<Item = &'a ShipState> {
        let id = self.id;
        game.ship_collection
            .values()
            .filter(move |ship| ship.player_id == id)
    }

    pub fn create_ship(&mut self, x: f64, y: f64) {
        let msg = StateMessage::CreateShip {
            ship: ShipState {
                player_id: self.id,
                position: (x, y).into(),
                ..Default::default()
            },
        };
        if let Err(err) = self.actions.send(msg).context(file!()) {
            error!("Error sending message: {}", err)
        };
    }

    pub fn next_message(&self) -> Option<StateMessage> {
        self.actions_buffer.try_recv().ok()
    }

    pub fn collect_messages(&self) -> Vec<StateMessage> {
        self.actions_buffer.try_iter().collect()
    }

    pub fn number_of_ships(&self, state: &ServerState) -> usize {
        return self.player_ships(state).count();
    }

    pub fn number_of_idle_ships(&self, state: &ServerState) -> usize {
        return self
            .player_ships(state)
            .filter(|ship| ship.speed.magnitude2() == 0.0)
            .count();
    }

    pub fn tick(&mut self, game_state: &ServerState) {
        for player_ship in self.moving_ships.values_mut() {
            let path = &mut player_ship.path;
            let ship = game_state
                .ship_collection
                .get(&ShipKey::new(player_ship.id, self.id));
            let ship = if let Some(ship) = ship {
                ship
            } else {
                player_ship.should_remove = true;
                continue;
            };
            loop {
                if let Some(&next) = path.first() {
                    let position: V2D = ship.position.into();
                    let direction = next - position;
                    let is_final_target = path.len() == 1;
                    let error_tolerance = if is_final_target { 1.0 } else { 5.0 };
                    let has_reached_goal = direction.magnitude() < error_tolerance;
                    if has_reached_goal {
                        path.remove(0);
                        if path.is_empty() {
                            if let Err(e) = self.actions.send(StateMessage::MoveShip {
                                player_id: self.id,
                                id: ship.id,
                                speed: (0.0, 0.0).into(),
                            }) {
                                log::error!("Error sending message: {:?}", e)
                            }
                            player_ship.should_remove = true;
                            break;
                        } else {
                            continue;
                        }
                    };
                    player_ship.target = Some(next);
                    let speed = direction.normalize() * BOAT_SPEED;
                    let has_speed_chanded = player_ship
                        .speed
                        .map(|s| is_different(s, speed))
                        .unwrap_or(true);

                    if !has_speed_chanded {
                        break;
                    }

                    player_ship.speed = Some(speed);

                    if let Err(e) = self.actions.send(StateMessage::MoveShip {
                        player_id: self.id,
                        id: ship.id,
                        speed: speed.into(),
                    }) {
                        log::error!("Error sending message: {:?}", e)
                    }
                    break;
                } else {
                    break;
                }
            }
        }

        self.moving_ships.retain(|_, ship| !ship.should_remove);
    }
}

fn is_different(a: V2D, b: V2D) -> bool {
    return (a - b).magnitude2() > 0.1;
}

fn unit_spiral_formation(n: usize, x: f64, y: f64, game: &ServerState) -> Vec<(f64, f64)> {
    let cell_size = 20.0;
    let spiral = SpiralSearch::new((0, 0));
    let mut v = Vec::with_capacity(n);
    for (x_i, y_i) in spiral {
        let x = x_i as f64 * cell_size + x;
        let y = y_i as f64 * cell_size + y;
        if game.game_map.is_allowed_place(x, y) {
            v.push((x, y));
        }
        if v.len() >= n {
            break;
        }
    }
    return v;
}

#[cfg(test)]
mod test {}
