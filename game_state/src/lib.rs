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
mod running_mode;
mod server_state;
mod spiral_search;
mod wasm_game;
mod world_gen;
mod ws_channel;
use wasm_bindgen::prelude::*;

pub use game_server::{GameServer, TICK_TIME};

#[wasm_bindgen(start)]
pub fn start() {
    //setup logger
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
}
