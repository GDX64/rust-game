use cgmath::InnerSpace;
use log::info;

use crate::{sparse_matrix::FV2D, ClientMessage, ServerState, ShipState};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PlayerShip {
    id: u64,
    position: FV2D,
    speed: FV2D,
    path: Vec<FV2D>,
}

pub struct Player {
    pub id: u64,
    ship_id: u64,
    moving_ships: HashMap<u64, PlayerShip>,
    actions: Vec<ClientMessage>,
}

impl Player {
    pub fn new(id: u64) -> Self {
        Player {
            id,
            moving_ships: HashMap::new(),
            actions: vec![],
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
        let server_ship = self.moving_ships.get(&ship_id)?;
        let path = game_state
            .game_map
            .find_path(server_ship.position, (x, y))?;
        info!("path: {:?}", path);
        let ship = PlayerShip {
            id: ship_id,
            position: server_ship.position.into(),
            speed: (0.0, 0.0).into(),
            path,
        };
        self.moving_ships.insert(ship_id, ship);
        Some(())
    }

    pub fn sync_with_server(&mut self, game_state: &ServerState) {
        game_state.get_ships().iter().for_each(|ship| {
            if ship.player_id != self.id {
                return;
            }
            if let Some(moving_ship) = self.moving_ships.get_mut(&ship.id) {
                moving_ship.position = ship.position.into();
            } else {
                self.moving_ships.insert(
                    ship.id,
                    PlayerShip {
                        id: ship.id,
                        path: vec![],
                        position: ship.position.into(),
                        speed: (0.0, 0.0).into(),
                    },
                );
            }
        });
    }

    pub fn create_ship(&mut self, x: f64, y: f64) {
        let msg = ClientMessage::CreateShip {
            ship: ShipState {
                id: self.next_id(),
                speed: (0.0, 0.0),
                player_id: self.id,
                position: (x, y),
            },
        };
        self.actions.push(msg);
    }

    fn next_id(&mut self) -> u64 {
        self.ship_id += 1;
        self.ship_id
    }

    pub fn take_actions(&mut self) -> Vec<ClientMessage> {
        self.actions.drain(..).collect()
    }

    pub fn tick(&mut self) {
        for ship in self.moving_ships.values_mut() {
            if let Some(next) = ship.path.first() {
                let direction = next - ship.position;
                if direction.magnitude() < 0.1 {
                    ship.path.remove(0);
                } else {
                    ship.speed = direction.normalize() * 2.0;
                }
                if ship.path.is_empty() {
                    ship.speed = (0.0, 0.0).into();
                }
                self.actions.push(ClientMessage::MoveShip {
                    player_id: self.id,
                    id: ship.id,
                    speed: ship.speed.into(),
                    position: ship.position.into(),
                });
            }
        }
    }
}
