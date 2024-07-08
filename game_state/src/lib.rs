mod server_state;
use core::panic;
use std::sync::mpsc::{Receiver, Sender};
mod running_mode;
use cgmath::Vector2;
pub use game_server::*;
use player::Player;
use running_mode::{OnlineData, RunningMode};
pub use server_state::*;
mod game_noise;
mod game_server;
mod interpolation;
mod player;
mod sparse_matrix;
mod world_gen;
use wasm_bindgen::prelude::*;
mod diffing;

#[wasm_bindgen(start)]
pub fn start() {
    //setup logger
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
}

#[wasm_bindgen]
pub struct GameWasmState {
    running_mode: RunningMode,
    player: Player,
    sender: Sender<ClientMessage>,
    receiver: Receiver<ClientMessage>,
    last_state: BroadCastState,
}

#[wasm_bindgen]
impl GameWasmState {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            running_mode: RunningMode::start_local(),
            player: Player::new(0, sender.clone()),
            receiver,
            sender,
            last_state: BroadCastState::new(),
        }
    }

    pub fn get_state_diff(&mut self) -> String {
        let current_state = self.running_mode.server_state().get_broadcast_state();
        let diff = self.last_state.diff(&current_state);
        self.last_state = current_state;
        let result = serde_json::to_string(&diff).unwrap_or("{}".to_string());
        result
    }

    pub fn shoot_with_all(&self) {
        self.player.shoot_with_all_ships();
    }

    pub fn start_local_server(&mut self) {
        self.running_mode = RunningMode::start_local();
        self.player = Player::new(self.running_mode.id(), self.sender.clone());
    }

    pub fn start_online(&mut self, on_data: OnlineData) {
        self.running_mode = RunningMode::Online(on_data);
        self.player = Player::new(self.running_mode.id(), self.sender.clone());
    }

    pub fn tick(&mut self) {
        self.player
            .sync_with_server(&self.running_mode.server_state());
        self.player.tick();
        while let Ok(action) = self.receiver.try_recv() {
            self.send_message(action);
        }
        self.running_mode.tick();
    }

    fn send_message(&mut self, msg: ClientMessage) {
        self.running_mode.send_message(msg);
    }

    pub fn action_create_ship(&mut self, x: f64, y: f64) {
        self.player.create_ship(x, y);
    }

    pub fn action_move_ship(&mut self, id: f64, x: f64, y: f64) {
        self.player
            .move_ship(&self.running_mode.server_state(), id as u64, x, y);
    }

    pub fn get_all_ships(&self) -> String {
        let ships: Vec<ShipState> = self.running_mode.server_state().get_ships();
        serde_json::to_string(&ships).unwrap_or("[]".to_string())
    }

    pub fn find_path(&self, xi: f64, yi: f64, xf: f64, yf: f64) -> Option<String> {
        let result = self
            .running_mode
            .server_state()
            .game_map
            .find_path(Vector2::new(xi, yi), Vector2::new(xf, yf))?;
        let result: Vec<(f64, f64)> = result.into_iter().map(|v| (v.x, v.y)).collect();
        serde_json::to_string(&result).ok()
    }

    pub fn map_size(&self) -> f64 {
        self.running_mode.server_state().game_map.dim
    }

    pub fn world_gen(&mut self) -> world_gen::WorldGen {
        return self.running_mode.server_state().world_gen.clone();
    }

    pub fn get_land_grid_value(&self, x: f64, y: f64) -> Option<f64> {
        let result = self.running_mode.server_state().game_map.get(x, y)?.0;
        Some(result)
    }

    pub fn get_land_value(&self, x: f64, y: f64) -> f64 {
        self.running_mode
            .server_state()
            .world_gen
            .get_land_value(x, y)
    }

    pub fn my_id(&self) -> f64 {
        self.running_mode.id() as f64
    }

    pub fn get_players(&self) -> String {
        let player: Vec<PlayerState> = self
            .running_mode
            .server_state()
            .players
            .values()
            .cloned()
            .collect();
        serde_json::to_string(&player).unwrap_or("[]".to_string())
    }
}
