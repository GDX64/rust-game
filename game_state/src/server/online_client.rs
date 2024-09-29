use crate::{
    utils::vectors::V2D,
    wasm_game::{GameMessage, ServerState},
};

use super::{local_client::Client, ws_channel::WSChannel};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    ws: WSChannel,
    id: u64,
    send_buffer: Vec<GameMessage>,
    start_position: V2D,
    receiver_buffer: Vec<GameMessage>,
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        OnlineClient {
            ws: WSChannel::new(url),
            id: 0,
            send_buffer: vec![],
            start_position: V2D::new(0.0, 0.0),
            receiver_buffer: vec![],
        }
    }

    async fn async_next(&mut self) -> Option<GameMessage> {
        if !self.receiver_buffer.is_empty() {
            return Some(self.receiver_buffer.remove(0));
        }
        let msg = self.ws.next().await;
        let msg = match msg {
            Some(msg) => msg,
            _ => return None,
        };
        let msg = GameMessage::from_arr_bytes(&msg);
        self.receiver_buffer = msg;
        if !self.receiver_buffer.is_empty() {
            Some(self.receiver_buffer.remove(0))
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<GameMessage> {
        if !self.receiver_buffer.is_empty() {
            return Some(self.receiver_buffer.remove(0));
        }
        let msg = self.ws.receive();
        let msg = match msg {
            Some(msg) => msg,
            _ => return None,
        };
        let msg = GameMessage::from_arr_bytes(&msg);
        self.receiver_buffer = msg;
        if !self.receiver_buffer.is_empty() {
            Some(self.receiver_buffer.remove(0))
        } else {
            None
        }
    }

    fn flush_send_buffer(&mut self) {
        self.ws.send(GameMessage::serialize_arr(&self.send_buffer));
        self.send_buffer.clear();
    }
}

impl Client for OnlineClient {
    fn send(&mut self, msg: GameMessage) {
        self.send_buffer.push(msg);
    }

    fn next_message(&mut self) -> Option<GameMessage> {
        self.next()
    }

    fn tick(&mut self, dt: f64) {
        self.flush_send_buffer();
    }

    fn server_state(&self) -> Option<&ServerState> {
        return None;
    }
}
