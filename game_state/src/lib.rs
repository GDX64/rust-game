mod server_state;
use core::panic;
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
use sparse_matrix::V2D;
use wasm_bindgen::prelude::*;
mod diffing;
mod ws_channel;

#[wasm_bindgen(start)]
pub fn start() {
    //setup logger
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
}

#[wasm_bindgen]
pub struct GameWasmState {
    running_mode: RunningMode,
    player: Player,
    last_state: BroadCastState,
    current_time: f64,
}

#[wasm_bindgen]
impl GameWasmState {
    pub fn new() -> Self {
        Self {
            running_mode: RunningMode::start_local(),
            player: Player::new(0),
            last_state: BroadCastState::new(),
            current_time: 0.0,
        }
    }

    pub fn get_state_diff(&mut self) -> String {
        let current_state = self.running_mode.server_state().get_broadcast_state();
        let diff = self.last_state.diff(&current_state);
        self.last_state = current_state;
        let result = serde_json::to_string(&diff).unwrap_or("{}".to_string());
        result
    }

    pub fn shoot_with_all(&self, x: f64, y: f64, camera_x: f64, camera_y: f64) {
        self.player
            .shoot_with_all_ships(&V2D::new(x, y), self.running_mode.server_state());
    }

    pub fn get_all_explosions(&self) -> JsValue {
        let explosions = self
            .running_mode
            .server_state()
            .explosions
            .values()
            .collect::<Vec<_>>();
        serde_wasm_bindgen::to_value(&explosions).unwrap_or_default()
    }

    pub fn start_local_server(&mut self) {
        self.running_mode = RunningMode::start_local();
        self.player = Player::new(self.running_mode.id());
    }

    pub fn start_online(&mut self, on_data: OnlineData) {
        self.running_mode = RunningMode::Online(on_data);
        self.player = Player::new(self.running_mode.id());
    }

    pub fn tick(&mut self, time: f64) {
        let dt = time - self.current_time;
        self.current_time = time;
        self.player.tick(&self.running_mode.server_state());
        while let Some(action) = self.player.next_message() {
            self.send_message(action);
        }
        self.running_mode.tick(dt);
    }

    pub fn get_all_bullets(&self) -> JsValue {
        let bullets = self.running_mode.server_state().get_bullets();
        serde_wasm_bindgen::to_value(&bullets).unwrap_or_default()
    }

    fn send_message(&mut self, msg: ClientMessage) {
        self.running_mode
            .send_game_message(GameMessage::ClientMessage(msg));
    }

    pub fn tile_size(&self) -> f64 {
        return self.running_mode.server_state().game_map.tile_size;
    }

    pub fn action_create_ship(&mut self, x: f64, y: f64) {
        self.player.create_ship(x, y);
    }

    pub fn action_move_ship(&mut self, id: f64, x: f64, y: f64) {
        self.player
            .move_ship(&self.running_mode.server_state(), id as u64, x, y);
    }

    pub fn add_bot(&mut self) {
        self.running_mode.send_game_message(GameMessage::AddBot)
    }

    pub fn remove_bot(&mut self) {
        self.running_mode.send_game_message(GameMessage::RemoveBot)
    }

    pub fn get_all_ships(&self) -> JsValue {
        let ships: Vec<ShipState> = self.running_mode.server_state().get_ships();
        serde_wasm_bindgen::to_value(&ships).unwrap_or_default()
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
