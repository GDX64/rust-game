use anyhow::Context;
use cgmath::InnerSpace;
use log::error;

use crate::{sparse_matrix::V2D, ClientMessage, ServerState, ShipKey, ShipState};
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

#[derive(Debug)]
pub struct PlayerShip {
    path: Vec<V2D>,
    id: u64,
    destroyed: bool,
}

pub struct Player {
    pub id: u64,
    ship_id: u64,
    moving_ships: HashMap<u64, PlayerShip>,
    actions: Sender<ClientMessage>,
    actions_buffer: Receiver<ClientMessage>,
}

impl Player {
    pub fn new(id: u64) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Player {
            id,
            moving_ships: HashMap::new(),
            actions: sender,
            actions_buffer: receiver,
            ship_id: 0,
        }
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
            .get(&ShipKey::new(self.id, ship_id))?;
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

    fn shoot_at(&self, ship_id: u64, x: f64, y: f64) {
        let msg = ClientMessage::Shoot {
            ship_id,
            player_id: self.id,
            target: (x, y),
        };
        if let Err(err) = self.actions.send(msg).context(file!()) {
            error!("Error sending message: {}", err)
        };
    }

    pub fn shoot_with_all_ships(&self, target: &V2D, _camera: &V2D, game_state: &ServerState) {
        self.moving_ships.iter().for_each(|(_, ship)| {
            if let Some(ship) = game_state
                .ship_collection
                .get(&ShipKey::new(self.id, ship.id))
            {
                self.shoot_at(ship.id, target.x, target.y);
            };
        });
    }

    pub fn create_ship(&mut self, x: f64, y: f64) {
        let msg = ClientMessage::CreateShip {
            ship: ShipState {
                id: self.next_id(),
                acceleration: (0.0, 0.0),
                speed: (0.0, 0.0),
                player_id: self.id,
                position: (x, y),
            },
        };
        if let Err(err) = self.actions.send(msg).context(file!()) {
            error!("Error sending message: {}", err)
        };
    }

    pub fn next_message(&self) -> Option<ClientMessage> {
        self.actions_buffer.try_recv().ok()
    }

    pub fn has_ships(&self, state: &ServerState) -> bool {
        return state
            .ship_collection
            .iter()
            .any(|(key, _)| key.player_id == self.id);
    }

    fn next_id(&mut self) -> u64 {
        self.ship_id += 1;
        self.ship_id
    }

    pub fn tick(&mut self, game_state: &ServerState) {
        for player_ship in self.moving_ships.values_mut() {
            let path = &mut player_ship.path;
            let ship = game_state
                .ship_collection
                .get(&ShipKey::new(self.id, player_ship.id));
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
                    if direction.magnitude() < 0.1 {
                        path.remove(0);
                        if path.is_empty() {
                            self.actions
                                .send(ClientMessage::MoveShip {
                                    player_id: self.id,
                                    id: ship.id,
                                    acceleration: (0.0, 0.0),
                                    speed: (0.0, 0.0),
                                    position: ship.position,
                                })
                                .expect("Error sending message");
                            break;
                        } else {
                            continue;
                        }
                    };
                    let acceleration = direction.normalize() * 2.0;
                    let speed: V2D = ship.speed.into();
                    self.actions
                        .send(ClientMessage::MoveShip {
                            player_id: self.id,
                            id: ship.id,
                            acceleration: acceleration.into(),
                            speed: speed.into(),
                            position: ship.position.into(),
                        })
                        .expect("Error sending message");
                    break;
                } else {
                    break;
                }
            }
        }
    }
}
