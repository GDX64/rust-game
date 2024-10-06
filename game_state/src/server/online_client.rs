use crate::wasm_game::{GameMessage, ServerState};

use super::{local_client::Client, ws_channel::WSChannel};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    send_buffer: Vec<GameMessage>,
    receiver: Option<Receiver<GameMessage>>,
    ws: Option<WSChannel>,
    url: String,
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        let mut client = OnlineClient {
            receiver: None,
            send_buffer: vec![],
            url: url.to_string(),
            ws: None,
        };
        client.reconnect(None);
        client
    }

    fn next(&mut self) -> Option<GameMessage> {
        self.receiver.as_mut()?.try_next().ok()?
    }

    fn flush_send_buffer(&mut self) -> Option<()> {
        self.ws
            .as_mut()?
            .send(GameMessage::serialize_arr(&self.send_buffer));
        self.send_buffer.clear();
        Some(())
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

    fn reconnect(&mut self, player_id: Option<u64>) {
        let url = if let Some(id) = player_id {
            format!("{}&player_id={}", self.url, id)
        } else {
            self.url.clone()
        };

        let (mut sender, receiver) = channel(100);

        let mut ws = WSChannel::new(&url);
        let mut channel_receiver = ws.receiver().expect("Failed to get receiver");
        wasm_bindgen_futures::spawn_local(async move {
            log::info!("Reconnecting to {}", url);
            loop {
                let ans = channel_receiver.next().await;
                match ans {
                    Some(msg) => {
                        let msg = GameMessage::from_arr_bytes(&msg);
                        msg.into_iter().for_each(|msg| {
                            match sender.try_send(msg) {
                                Err(e) => log::error!("Failed to send message: {:?}", e),
                                _ => (),
                            }
                        });
                    }
                    None => {
                        sender
                            .send(GameMessage::ConnectionDown)
                            .await
                            .expect("Failed to send");
                        break;
                    }
                }
            }
        });
        self.receiver = Some(receiver);
        self.ws = Some(ws);
    }
}
