mod server_state;
use core::panic;
use std::borrow::BorrowMut;

use cgmath::Vector2;
pub use game_server::*;
use log::{error, info};
pub use server_state::*;
mod game_noise;
mod game_server;
mod interpolation;
mod sparse_matrix;
mod world_gen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    //setup logger
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
}

#[wasm_bindgen]
pub struct GameWasmState {
    server_state: ServerState,
    running_mode: RunningMode,
    incremental_id: u64,
}

enum RunningMode {
    Local(game_server::GameServer),
    None,
}

#[wasm_bindgen]
impl GameWasmState {
    pub fn new() -> Self {
        Self {
            server_state: ServerState::new(),
            running_mode: RunningMode::None,
            incremental_id: 0,
        }
    }

    pub async fn start_local_server(&mut self) {
        let player_message = GameMessage::NewConnection;
        let mut game = game_server::GameServer::new();

        let id = game
            .on_message(player_message)
            .expect("should be possible to start local server");
        if let GameServerMessageResult::PlayerID(id) = id {
            self.server_state.on_message(ClientMessage::MarkMyID { id })
        } else {
            panic!("should be possible to start local server");
        }

        self.running_mode = RunningMode::Local(game);

        info!("Local server started")
    }

    pub fn tick(&mut self) {
        self.poll_player_messages();
        match &mut self.running_mode {
            RunningMode::Local(game) => {
                game.on_message(GameMessage::Tick)
                    .expect("should be possible to tick");
            }
            RunningMode::None => {}
        };
    }

    fn poll_player_messages(&mut self) {
        let server = self.server_state.borrow_mut();
        match &mut self.running_mode {
            RunningMode::Local(game) => {
                let my_id = server.my_id.unwrap_or(0);
                game.messages_to_send.drain(..).for_each(|(id, msg)| {
                    if id == my_id {
                        if let Err(err) = server.on_string_message(msg.clone()) {
                            error!("Error processing message: {:?}", err)
                        }
                    }
                });
            }
            RunningMode::None => {}
        }
    }

    fn send_message(&mut self, msg: ClientMessage) {
        match self.running_mode {
            RunningMode::Local(ref mut game) => {
                let msg = serde_json::to_string(&msg).expect("should be possible to serialize");
                let msg = GameMessage::ClientMessage(msg);
                match game.on_message(msg) {
                    Err(err) => error!("Error sending message: {:?}", err),
                    Ok(_) => {}
                }
            }
            RunningMode::None => {}
        }
    }

    pub fn action_create_ship(&mut self, x: f64, y: f64) {
        let msg = ClientMessage::CreateShip {
            ship: ShipState {
                id: self.next_id(),
                player_id: self.server_state.my_id.unwrap_or(0),
                position: (x, y),
            },
        };
        self.send_message(msg);
    }

    pub fn action_move_ship(&mut self, id: u64, x: f64, y: f64) {
        let msg = ClientMessage::MoveShip {
            player_id: self.server_state.my_id.unwrap_or(0),
            id,
            position: (x, y),
        };
        self.send_message(msg);
    }

    fn next_id(&mut self) -> u64 {
        self.incremental_id += 1;
        self.incremental_id
    }

    pub fn get_all_ships(&self) -> String {
        let ships: Vec<ShipState> = self.server_state.get_ships();
        serde_json::to_string(&ships).unwrap_or("[]".to_string())
    }

    pub fn find_path(&self, xi: f64, yi: f64, xf: f64, yf: f64) -> Option<String> {
        let result = self
            .server_state
            .game_map
            .find_path(Vector2::new(xi, yi), Vector2::new(xf, yf))?;
        let result: Vec<(f64, f64)> = result.into_iter().map(|v| (v.x, v.y)).collect();
        serde_json::to_string(&result).ok()
    }

    pub fn map_size(&self) -> f64 {
        self.server_state.game_map.dim
    }

    pub fn world_gen(&mut self) -> world_gen::WorldGen {
        return self.server_state.world_gen.clone();
    }

    pub fn get_land_grid_value(&self, x: f64, y: f64) -> Option<f64> {
        let result = self.server_state.game_map.get(x, y)?.0;
        Some(result)
    }

    pub fn get_land_value(&self, x: f64, y: f64) -> f64 {
        self.server_state.world_gen.get_land_value(x, y)
    }

    pub fn my_id(&self) -> Option<f64> {
        self.server_state.my_id.map(|id| id as f64)
    }

    pub fn get_players(&self) -> String {
        let player: Vec<PlayerState> = self.server_state.players.values().cloned().collect();
        serde_json::to_string(&player).unwrap_or("[]".to_string())
    }
}
