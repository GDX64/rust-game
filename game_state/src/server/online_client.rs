use crate::wasm_game::{GameMessage, ServerState};

use super::{local_client::Client, ws_channel::WSChannel};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    ws: WSChannel,
    send_buffer: Vec<GameMessage>,
    receiver_buffer: Vec<GameMessage>,
    url: String,
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        OnlineClient {
            ws: WSChannel::new(url),
            send_buffer: vec![],
            receiver_buffer: vec![],
            url: url.to_string(),
        }
    }

    // async fn async_next(&mut self) -> Option<GameMessage> {
    //     if !self.receiver_buffer.is_empty() {
    //         return Some(self.receiver_buffer.remove(0));
    //     }
    //     let msg = self.ws.next().await;
    //     let msg = match msg {
    //         Some(msg) => msg,
    //         _ => return None,
    //     };
    //     let msg = GameMessage::from_arr_bytes(&msg);
    //     self.receiver_buffer = msg;
    //     if !self.receiver_buffer.is_empty() {
    //         Some(self.receiver_buffer.remove(0))
    //     } else {
    //         None
    //     }
    // }

    fn next(&mut self) -> Option<GameMessage> {
        if !self.receiver_buffer.is_empty() {
            return Some(self.receiver_buffer.remove(0));
        }
        if self.ws.is_offline() {
            return Some(GameMessage::ConnectionDown);
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

    fn tick(&mut self, _dt: f64) {
        self.flush_send_buffer();
    }

    fn server_state(&self) -> Option<&ServerState> {
        return None;
    }

    fn reconnect(&mut self, player_id: u64) {
        if !self.ws.is_connecting() {
            let url = format!("{}&player_id={}", self.url, player_id);
            self.ws = WSChannel::new(&url);
            log::info!("Reconnecting to {}", url);
        }
    }
}
