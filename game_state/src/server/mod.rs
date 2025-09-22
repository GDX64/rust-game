use futures::channel::mpsc::Receiver;

use crate::server::game_server::GameMessage;

pub mod game_server;
#[cfg(target_arch = "wasm32")]
pub mod local_client;
#[cfg(target_arch = "wasm32")]
pub mod ws_channel;

pub mod online_client;
pub mod running_mode;

pub trait Client: Send {
    fn send(&mut self, msg: GameMessage);
    fn tick(&mut self, dt: f64);
    fn take_receiver(&mut self) -> Option<Receiver<GameMessage>>;
    fn reconnect(&mut self);
}
