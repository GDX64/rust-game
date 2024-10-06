pub mod game_server;
#[cfg(target_arch = "wasm32")]
pub mod local_client;
#[cfg(target_arch = "wasm32")]
pub mod online_client;
#[cfg(target_arch = "wasm32")]
pub mod running_mode;
#[cfg(target_arch = "wasm32")]
mod ws_channel;
