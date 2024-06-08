mod server_state;
pub use server_state::*;
mod game_noise;
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
