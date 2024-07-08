use anyhow::Context;
use cgmath::InnerSpace;
use log::{error, info};

use crate::{sparse_matrix::V2D, ClientMessage, ServerState, ShipState};
use std::{collections::HashMap, sync::mpsc::Sender};

#[derive(Debug)]
pub struct PlayerShip {
    id: u64,
    position: V2D,
    speed: V2D,
    path: Vec<V2D>,
}

pub struct Player {
    pub id: u64,
    ship_id: u64,
    moving_ships: HashMap<u64, PlayerShip>,
    actions: Sender<ClientMessage>,
}

impl Player {
    pub fn new(id: u64, sender: Sender<ClientMessage>) -> Self {
        Player {
            id,
            moving_ships: HashMap::new(),
            actions: sender,
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

    pub fn shoot_with_all_ships(&self) {
        self.moving_ships.iter().for_each(|(id, ship)| {
            let speed_direction = ship.speed.normalize();
            let speed_90_deg = V2D::new(-speed_direction.y, speed_direction.x);
            let target = ship.position + speed_90_deg * 30.0;
            self.shoot_at(*id, target.x, target.y);
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
        if let Err(err) = self.actions.send(msg).context(file!()) {
            error!("Error sending message: {}", err)
        };
    }

    fn next_id(&mut self) -> u64 {
        self.ship_id += 1;
        self.ship_id
    }

    pub fn tick(&mut self) {
        for ship in self.moving_ships.values_mut() {
            if let Some(next) = ship.path.first() {
                let direction = next - ship.position;
                if direction.magnitude() < 0.1 {
                    ship.path.remove(0);
                } else {
                    ship.speed = direction.normalize() / 2.0;
                }
                if ship.path.is_empty() {
                    ship.speed = (0.0, 0.0).into();
                }
                if let Err(err) = self
                    .actions
                    .send(ClientMessage::MoveShip {
                        player_id: self.id,
                        id: ship.id,
                        speed: ship.speed.into(),
                        position: ship.position.into(),
                    })
                    .context(file!())
                {
                    error!("Error sending message: {}", err)
                }
            }
        }
    }
}
