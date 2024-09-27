use core::panic;

use crate::utils::vectors::V2D;
use crate::wasm_game::{ServerState, StateMessage, TICK_TIME};
use crate::ws_channel::WSChannel;
use crate::{game_server, wasm_game::GameMessage};
use futures::channel::mpsc::{channel, Receiver};
use futures::StreamExt;
use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    ws: WSChannel,
    game_state: ServerState,
    id: u64,
    frame_acc: f64,
    frame_buffer: Vec<Vec<StateMessage>>,
    send_buffer: Vec<GameMessage>,
    start_position: V2D,
    receiver_buffer: Vec<GameMessage>,
}

pub trait Client {
    fn send(&mut self, msg: GameMessage);
    fn tick(&mut self, dt: f64);
    fn get_id(&self) -> u64;
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        OnlineClient {
            ws: WSChannel::new(url),
            game_state: ServerState::new(),
            id: 0,
            frame_acc: 0.0,
            frame_buffer: vec![],
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

    pub async fn init(&mut self) -> JsValue {
        loop {
            match self.async_next().await {
                Some(GameMessage::PlayerCreated { id, x, y }) => {
                    info!("My ID is: {}", id);
                    self.id = id;
                    self.start_position = V2D::new(x, y);
                    self.send(GameMessage::AskBroadcast { player: id });
                    return serde_wasm_bindgen::to_value(&vec![x, y]).unwrap();
                }
                _ => {}
            }
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

    fn get_id(&self) -> u64 {
        self.id
    }

    fn tick(&mut self, dt: f64) {
        self.flush_send_buffer();
        loop {
            let msg = self.next();
            let msg = match msg {
                Some(msg) => msg,
                _ => break,
            };
            match msg {
                GameMessage::FrameMessage(msg) => {
                    self.frame_buffer.insert(0, msg);
                }
                _ => {}
            }
        }

        self.frame_acc += dt;
        let completed_frames = (self.frame_acc / TICK_TIME).round();
        self.frame_acc -= (completed_frames) * TICK_TIME;

        for _ in 0..completed_frames as usize {
            loop {
                if let Some(frame) = self.frame_buffer.pop() {
                    frame
                        .into_iter()
                        .for_each(|msg| self.game_state.on_message(msg));
                }
                if self.frame_buffer.len() < 10 {
                    break;
                }
            }
        }
    }
}

#[wasm_bindgen]
pub struct LocalClient {
    game: game_server::GameServer,
    receiver: Receiver<Vec<u8>>,
    state: ServerState,
    id: u64,
    start_position: V2D,
}

#[wasm_bindgen]
impl LocalClient {
    pub fn new() -> LocalClient {
        let (sender, receiver) = channel(100);
        let mut game = game_server::GameServer::new();
        let player_id = game.new_connection(sender, None);
        info!("Local server started");
        LocalClient {
            game,
            receiver,
            state: ServerState::new(),
            id: player_id,
            start_position: V2D::new(0.0, 0.0),
        }
    }

    pub async fn init(&mut self) -> JsValue {
        self.game.flush_send_buffers();
        while let Some(msg) = self.receiver.next().await {
            let game_message = GameMessage::from_arr_bytes(&msg);
            for msg in game_message.into_iter() {
                match msg {
                    GameMessage::PlayerCreated { id, x, y } => {
                        self.id = id;
                        self.start_position = V2D::new(x, y);
                        log::info!("My ID is: {}", id);
                        return serde_wasm_bindgen::to_value(&vec![x, y]).unwrap();
                    }
                    _ => {}
                }
            }
        }
        log::error!("Failed to connect to server");
        panic!("Failed to connect to server");
    }
}

impl Client for LocalClient {
    fn send(&mut self, msg: GameMessage) {
        self.game.on_message(GameMessage::serialize_arr(&vec![msg]));
    }

    fn tick(&mut self, dt: f64) {
        self.game.tick(dt);
        while let Ok(Some(msg)) = self.receiver.try_next() {
            let game_message = GameMessage::from_arr_bytes(&msg);
            for msg in game_message.into_iter() {
                match msg {
                    GameMessage::FrameMessage(msg) => {
                        msg.into_iter().for_each(|msg| self.state.on_message(msg));
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_id(&self) -> u64 {
        self.id
    }
}

pub enum RunningMode {
    Local(LocalClient),
    Online(OnlineClient),
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        match self {
            RunningMode::Local(data) => &data.state,
            RunningMode::Online(data) => &data.game_state,
        }
    }

    pub fn start_position(&self) -> V2D {
        match self {
            RunningMode::Local(data) => data.start_position,
            RunningMode::Online(data) => data.start_position,
        }
    }

    pub fn start_local(client: LocalClient) -> RunningMode {
        RunningMode::Local(client)
    }

    pub fn tick(&mut self, dt: f64) {
        match self {
            RunningMode::Local(client) => {
                client.tick(dt);
            }
            RunningMode::Online(data) => {
                data.tick(dt);
            }
        };
    }

    pub fn clear_flags(&mut self) {
        match self {
            RunningMode::Local(data) => {
                data.state.clear_flags();
            }
            RunningMode::Online(data) => {
                data.game_state.clear_flags();
            }
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local(data) => data.id,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn send_game_message(&mut self, msg: GameMessage) {
        match self {
            RunningMode::Local(ref mut data) => {
                data.game.on_message(GameMessage::serialize_arr(&vec![msg]));
            }
            RunningMode::Online(data) => {
                data.send(msg);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::wasm_game::GameMessage;

    #[test]
    fn running_mode() {
        let client = super::LocalClient::new();
        let mut local = super::RunningMode::start_local(client);
        local.send_game_message(GameMessage::AddBot);
        local.send_game_message(GameMessage::AddBot);
        for _ in 0..1000 {
            local.tick(0.016)
        }
        match local {
            super::RunningMode::Local(data) => {
                assert_eq!(
                    data.game.game_state.ship_collection,
                    data.state.ship_collection
                );
            }
            _ => panic!("Expected local mode"),
        }
    }
}
