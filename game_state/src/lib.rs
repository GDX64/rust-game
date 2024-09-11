mod boidlike;
mod bot_player;
mod bullet;
mod diffing;
mod game_map;
mod game_noise;
mod game_server;
mod hashgrid;
mod interpolation;
mod player;
mod player_state;
mod running_mode;
mod server_state;
mod spiral_search;
mod wasm_game;
mod world_gen;
mod ws_channel;
pub use game_server::{GameServer, TICK_TIME};
use std::sync::OnceLock;
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
}
