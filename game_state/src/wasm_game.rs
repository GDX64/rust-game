use crate::game_map::V2D;
pub use crate::game_server::*;
use crate::player::Player;
use crate::running_mode::{OnlineClient, RunningMode};
pub use crate::server_state::*;
use crate::world_gen::WorldGenConfig;
use cgmath::Vector2;
use core::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameWasmState {
    running_mode: RunningMode,
    player: Player,
    pub current_time: f64,
}

#[wasm_bindgen]
impl GameWasmState {
    pub fn new() -> Self {
        Self {
            running_mode: RunningMode::start_local(),
            player: Player::new(0),
            current_time: 0.0,
        }
    }

    pub fn action_shoot_at(&mut self, x: f64, y: f64) {
        self.player
            .shoot_at(&V2D::new(x, y), self.running_mode.server_state());
    }

    pub fn has_map_changed(&self) -> bool {
        self.running_mode.server_state().flags.map_changed
    }

    pub fn clear_flags(&mut self) {
        self.running_mode.clear_flags();
    }

    pub fn uint_terrain(&self) -> Vec<i16> {
        let terrain = self.running_mode.server_state().minimap();
        return terrain;
    }

    pub fn shoot_radius(&self) -> f64 {
        self.player.shoot_radius
    }

    pub fn change_shoot_radius(&mut self, r: f64) {
        self.player.change_shoot_radius(r);
    }

    pub fn gen_config(&self) -> WorldGenConfig {
        self.running_mode.server_state().world_gen.config.clone()
    }

    pub fn min_max_height(&self) -> Vec<f64> {
        self.running_mode.server_state().world_gen.min_max_height()
    }

    pub fn add_bot_ship_at(&mut self, x: f64, y: f64) {
        self.running_mode
            .send_game_message(GameMessage::AddBotShipAt(x, y));
    }

    pub fn shoot_error_margin(&self, x: f64, y: f64) -> Option<f64> {
        self.player
            .shoot_error_margin((x, y).into(), self.running_mode.server_state())
    }

    pub fn get_selected_ships(&self) -> JsValue {
        let ships = &self.player.selected_ships;
        serde_wasm_bindgen::to_value(ships).unwrap_or_default()
    }

    pub fn auto_shoot(&mut self) {
        self.player.auto_shoot(self.running_mode.server_state());
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

    pub fn start_online(&mut self, on_data: OnlineClient) {
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

    pub fn change_error(&mut self, err: f64) {
        self.send_message(StateMessage::GameConstants {
            constants: GameConstants {
                wind_speed: (0.0, 0.0, 0.0),
                err_per_m: err,
            },
        });
    }

    pub fn move_selected_ships(&mut self, x: f64, y: f64) {
        self.player
            .move_selected_ships(&self.running_mode.server_state(), x, y);
    }

    pub fn action_clear_selected(&mut self) {
        self.player.clear_selected_ships();
    }

    pub fn action_selec_ship(&mut self, id: f64) {
        self.player
            .selec_ship(id as u64, &self.running_mode.server_state());
    }

    pub fn get_all_bullets(&self) -> JsValue {
        let bullets = self
            .running_mode
            .server_state()
            .get_bullets()
            .into_iter()
            .map(|b| b.snapshot())
            .collect::<Vec<_>>();
        serde_wasm_bindgen::to_value(&bullets).unwrap_or_default()
    }

    fn send_message(&mut self, msg: StateMessage) {
        self.running_mode
            .send_game_message(GameMessage::InputMessage(msg));
    }

    pub fn tile_size(&self) -> f64 {
        return self.running_mode.server_state().game_map.tile_size;
    }

    pub fn action_create_ship(&mut self, x: f64, y: f64) {
        self.player.create_ship(x, y);
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

    pub fn get_land_grid_value(&self, x: f64, y: f64) -> Option<f64> {
        let result = self
            .running_mode
            .server_state()
            .game_map
            .get(x, y)?
            .height();
        Some(result)
    }

    pub fn all_island_data(&self) -> JsValue {
        let islands = self.running_mode.server_state().all_islands();
        serde_wasm_bindgen::to_value(&islands).unwrap_or_default()
    }

    pub fn island_owners(&self) -> JsValue {
        let owners = &self.running_mode.server_state().island_dynamic;
        serde_wasm_bindgen::to_value(&owners).unwrap_or_default()
    }

    pub fn island_at(&self, x: f64, y: f64) -> JsValue {
        let island = self.running_mode.server_state().island_at(x, y);
        serde_wasm_bindgen::to_value(&island).unwrap_or_default()
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
