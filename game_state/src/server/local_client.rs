use crate::wasm_game::{GameMessage, ServerState};
use futures::channel::mpsc::{channel, Receiver};
use log::info;
pub use wasm_bindgen::prelude::*;

use super::game_server;

pub trait Client {
    fn send(&mut self, msg: GameMessage);
    fn tick(&mut self, dt: f64);
    fn next_message(&mut self) -> Option<GameMessage>;
    fn server_state(&self) -> Option<&ServerState>;
}

#[wasm_bindgen]
pub struct LocalClient {
    game: game_server::GameServer,
    receiver: Receiver<Vec<u8>>,
    receive_buffer: Vec<GameMessage>,
}

#[wasm_bindgen]
impl LocalClient {
    pub fn new() -> LocalClient {
        let (sender, receiver) = channel(100);
        let mut game = game_server::GameServer::new();
        game.new_connection(sender, None);
        info!("Local server started");
        LocalClient {
            game,
            receiver,
            receive_buffer: vec![],
        }
    }
}

impl Client for LocalClient {
    fn send(&mut self, msg: GameMessage) {
        self.game.on_message(GameMessage::serialize_arr(&vec![msg]));
    }

    fn tick(&mut self, dt: f64) {
        self.game.tick(dt);
    }

    fn server_state(&self) -> Option<&ServerState> {
        Some(&self.game.game_state)
    }

    fn next_message(&mut self) -> Option<GameMessage> {
        if !self.receive_buffer.is_empty() {
            return Some(self.receive_buffer.remove(0));
        }
        match self.receiver.try_next() {
            Ok(Some(msg)) => {
                let game_message = GameMessage::from_arr_bytes(&msg);
                for msg in game_message.into_iter() {
                    self.receive_buffer.push(msg);
                }
                return self.next_message();
            }
            _ => None,
        }
    }
}
