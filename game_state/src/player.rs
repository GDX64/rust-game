use anyhow::Context;
use cgmath::InnerSpace;
use log::error;

use crate::{game_map::V2D, ServerState, ShipKey, ShipState, StateMessage};
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

const BOAT_ACC: f64 = 30.0;

#[derive(Debug)]
pub struct PlayerShip {
    path: Vec<V2D>,
    id: u64,
    destroyed: bool,
}

pub struct Player {
    pub id: u64,
    moving_ships: HashMap<u64, PlayerShip>,
    pub selected_ships: Vec<u64>,
    actions: Sender<StateMessage>,
    actions_buffer: Receiver<StateMessage>,
    pub rng: fastrand::Rng,
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
            rng: fastrand::Rng::with_seed(0),
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
        let formation = unit_formation(self.selected_ships.len(), x, y);
        self.selected_ships
            .clone()
            .iter()
            .zip(formation)
            .for_each(|(&ship_id, (x, y))| {
                self.move_ship(game_state, ship_id, x, y);
            });
    }

    pub fn clear_selected_ships(&mut self) {
        self.selected_ships.clear();
    }

    pub fn selec_ship(&mut self, ship_id: u64, game: &ServerState) {
        self.selected_ships.push(ship_id);
        self.selected_ships
            .retain(|&id| game.get_ship(id, self.id).is_some())
    }

    pub fn move_ship(
        &mut self,
        game_state: &ServerState,
        ship_id: u64,
        x: f64,
        y: f64,
    ) -> Option<()> {
        let server_ship = game_state
            .ship_collection
            .get(&ShipKey::new(ship_id, self.id))?;
        let path = game_state
            .game_map
            .find_path(server_ship.position, (x, y))?;

        //the fist one is already the current position
        let path = path[1..].to_vec();

        let ship = PlayerShip {
            path,
            id: ship_id,
            destroyed: false,
        };

        self.moving_ships.insert(ship_id, ship);
        Some(())
    }

    pub fn shoot_at_with(&self, ship_id: u64, x: f64, y: f64) {
        let msg = StateMessage::Shoot {
            ship_id,
            player_id: self.id,
            target: (x, y),
        };
        if let Err(err) = self.actions.send(msg).context(file!()) {
            error!("Error sending message: {}", err)
        };
    }

    pub fn shoot_at(&self, target: &V2D, game_state: &ServerState) {
        let selected_number = self.selected_ships.len() / 2;
        self.shooting_ships(game_state)
            .take(selected_number)
            .for_each(|ship| {
                self.shoot_at_with(ship.id, target.x, target.y);
            });
    }

    fn rand_enemies<'a>(&mut self, game_state: &'a ServerState) -> Vec<&'a ShipState> {
        let mut enemies = self.enemies(game_state).collect::<Vec<_>>();
        self.rng.shuffle(&mut enemies);
        enemies
    }

    fn enemies<'a>(&self, game_state: &'a ServerState) -> impl Iterator<Item = &'a ShipState> {
        let id = self.id;
        game_state
            .ship_collection
            .values()
            .filter(move |ship| ship.player_id != id)
    }

    pub fn select_all(&mut self, game_state: &ServerState) {
        self.selected_ships = self.player_ships(game_state).map(|ship| ship.id).collect();
    }

    pub fn auto_shoot(&mut self, game_state: &ServerState) {
        let enemies = self.rand_enemies(game_state).into_iter();
        self.shooting_ships(game_state)
            .zip(enemies)
            .for_each(|(ship, enemy)| {
                self.shoot_at_with(ship.id, enemy.position.0, enemy.position.1);
            });
    }

    fn shooting_ships<'a>(&'a self, game: &'a ServerState) -> impl Iterator<Item = &'a ShipState> {
        self.player_ships(game).filter(|ship| {
            let is_selected = self.selected_ships.contains(&ship.id);
            let can_shoot = ship.find_available_cannon(game.current_time).is_some();
            return is_selected && can_shoot;
        })
    }

    pub fn shoot_error_margin(&self, target: V2D, game: &ServerState) -> Option<f64> {
        let mut ships = self.shooting_ships(game);
        let ship = ships.next()?;
        let error = game
            .game_constants
            .error_margin(ship.position.into(), target);
        return error;
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
                position: (x, y),
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

    pub fn tick(&mut self, game_state: &ServerState) {
        for player_ship in self.moving_ships.values_mut() {
            let path = &mut player_ship.path;
            let ship = game_state
                .ship_collection
                .get(&ShipKey::new(player_ship.id, self.id));
            let ship = if let Some(ship) = ship {
                ship
            } else {
                player_ship.destroyed = true;
                continue;
            };
            loop {
                if let Some(next) = path.first() {
                    let position: V2D = ship.position.into();
                    let direction = next - position;
                    if direction.magnitude() < 1.0 {
                        path.remove(0);
                        if path.is_empty() {
                            if let Err(e) = self.actions.send(StateMessage::MoveShip {
                                player_id: self.id,
                                id: ship.id,
                                acceleration: (0.0, 0.0),
                            }) {
                                log::error!("Error sending message: {:?}", e)
                            }
                            break;
                        } else {
                            continue;
                        }
                    };
                    let acceleration = direction.normalize() * BOAT_ACC;
                    if let Err(e) = self.actions.send(StateMessage::MoveShip {
                        player_id: self.id,
                        id: ship.id,
                        acceleration: acceleration.into(),
                    }) {
                        log::error!("Error sending message: {:?}", e)
                    }
                    break;
                } else {
                    break;
                }
            }
        }

        self.moving_ships.retain(|_, ship| !ship.destroyed);
    }
}

fn unit_formation(n: usize, x: f64, y: f64) -> Vec<(f64, f64)> {
    let cell_size = 20.0;
    let sqrt = (n as f64).sqrt();
    let cols = sqrt;
    let rows = (n as f64 / sqrt).ceil();

    let max_x = (cols - 1.0) * cell_size;
    let max_y = (rows - 1.0) * cell_size;

    let x = x - max_x / 2.0;
    let y = y - max_y / 2.0;

    let sqrt = sqrt as usize;

    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let x = x + (i % sqrt) as f64 * cell_size;
        let y = y + (i / sqrt) as f64 * cell_size;
        positions.push((x, y));
    }
    positions
}

#[cfg(test)]
mod test {
    #[test]
    fn test_formation_even() {
        let formation = super::unit_formation(4, 0.0, 0.0);
        let expected = vec![(-10.0, -10.0), (10.0, -10.0), (-10.0, 10.0), (10.0, 10.0)];
        assert_eq!(formation, expected);
    }
}
