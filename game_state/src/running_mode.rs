use crate::{game_server, ClientMessage, GameMessage, ServerState};
use log::info;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct OnlineData {
    sender: js_sys::Function,
    game_state: ServerState,
    id: u64,
    ws_sender: WSChannelSender,
    receiver: Receiver<String>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WSChannelSender {
    sender: Sender<String>,
}

#[wasm_bindgen]
impl WSChannelSender {
    pub fn send(&self, msg: String) {
        self.sender
            .send(msg)
            .expect("could not send WSChannelSender");
    }
}

#[wasm_bindgen]
impl OnlineData {
    pub fn new(sender: js_sys::Function) -> OnlineData {
        let (channel_sender, channel_receiver) = std::sync::mpsc::channel();
        let ws_sender = WSChannelSender {
            sender: channel_sender.clone(),
        };
        OnlineData {
            sender,
            ws_sender,
            game_state: ServerState::new(),
            id: 0,
            receiver: channel_receiver,
        }
    }

    pub fn ws_sender(&self) -> WSChannelSender {
        self.ws_sender.clone()
    }
}

impl OnlineData {
    pub fn send(&self, msg: GameMessage) {
        let msg = JsValue::from_str(&msg.to_string());
        self.sender
            .call1(&JsValue::null(), &msg)
            .expect("should be possible to call");
    }

    pub fn tick(&mut self) {
        self.receiver.try_iter().for_each(|msg| {
            let msg = GameMessage::from_string(msg);
            match msg {
                GameMessage::MyID(id) => {
                    info!("My ID is: {}", id);
                    self.id = id;
                }
                GameMessage::ClientMessage(msg) => {
                    self.game_state.on_message(msg);
                }
                _ => {}
            }
        });
        self.game_state.tick(0.016);
    }
}

pub enum RunningMode {
    Local(game_server::GameServer),
    Online(OnlineData),
    None(game_server::GameServer),
}

impl RunningMode {
    pub fn none() -> RunningMode {
        RunningMode::None(game_server::GameServer::new())
    }

    pub fn server_state(&self) -> &ServerState {
        match self {
            RunningMode::Local(game) => &game.game_state,
            RunningMode::None(game) => &game.game_state,
            RunningMode::Online(data) => &data.game_state,
        }
    }

    pub fn start_local() -> RunningMode {
        let mut game = game_server::GameServer::new();
        game.new_connection(0);
        info!("Local server started");
        return RunningMode::Local(game);
    }

    pub fn tick(&mut self) {
        match self {
            RunningMode::Local(game) => {
                game.tick(0.016);
                game.messages_to_send.drain(..);
            }
            RunningMode::None(_) => {}
            RunningMode::Online(data) => {
                data.tick();
            }
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local(_) => 0,
            RunningMode::None(_) => 0,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn send_message(&mut self, msg: ClientMessage) {
        match self {
            RunningMode::Local(ref mut game) => {
                game.on_message(msg);
            }
            RunningMode::None(_) => {}
            RunningMode::Online(data) => {
                let msg = GameMessage::ClientMessage(msg);
                info!("Sending message: {:?}", msg);
                data.send(msg);
            }
        }
    }
}
