mod server_state;
use cgmath::Vector2;
pub use game_server::*;
pub use server_state::*;
mod game_noise;
mod game_server;
mod interpolation;
mod sparse_matrix;
mod world_gen;

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub struct GameWasmState {
    server_state: ServerState,
}

#[wasm_bindgen]
impl GameWasmState {
    pub fn new() -> Self {
        Self {
            server_state: ServerState::new(),
        }
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

    pub fn on_string_message(&mut self, msg: String) -> Option<bool> {
        self.server_state.on_string_message(msg).ok()?;
        Some(true)
    }

    pub fn get_players(&self) -> String {
        let player: Vec<PlayerState> = self.server_state.players.values().cloned().collect();
        serde_json::to_string(&player).unwrap_or("[]".to_string())
    }
}

#[wasm_bindgen]
struct MessageCreator {}

#[wasm_bindgen]
impl MessageCreator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_player(&self) -> String {
        todo!()
    }
}
