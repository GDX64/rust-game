mod boidlike;
mod bot_player;
mod bullet;
mod executor;
mod game_map;
mod game_noise;
mod hashgrid;
mod island;
mod player;
mod player_state;
mod server;
mod server_state;
mod ship;
mod utils;
mod world_gen;
pub use bot_player::*;
pub use executor::*;
pub use player_state::PlayerState;
pub use server::game_server::{DBStatsMessage, GameMessage, GameServer, TICK_TIME};
pub use server::online_client::*;
pub use server::running_mode::*;
use std::sync::OnceLock;
#[cfg(target_arch = "wasm32")]
mod wasm_game;
use wasm_bindgen::prelude::*;

const FLAG_NAMES: &'static str = include_str!("../assets/flagnames.txt");
type FlagSet = Vec<&'static str>;
static ONCE_FLAGS: OnceLock<FlagSet> = OnceLock::new();

pub fn get_flag_names() -> &'static FlagSet {
    ONCE_FLAGS.get_or_init(|| FLAG_NAMES.lines().collect())
}

#[wasm_bindgen(start)]
pub fn start() {
    //setup logger
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    use std::panic;
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    log::info!("WASM module initialized");
}
