use crate::{game_server, ClientMessage, GameMessage, MessageToSend, ServerState};
use futures::channel::mpsc::channel;
use futures::channel::mpsc::{Receiver, Sender};
use futures::StreamExt;
use log::info;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct OnlineData {
    sender: js_sys::Function,
    game_state: ServerState,
    id: u64,
    ws_sender: WSChannelSender,
    receiver: Receiver<Vec<u8>>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WSChannelSender {
    sender: Sender<Vec<u8>>,
}

#[wasm_bindgen]
impl WSChannelSender {
    pub fn send(&mut self, msg: Vec<u8>) {
        self.sender
            .try_send(msg)
            .expect("could not send WSChannelSender");
    }
}

#[wasm_bindgen]
impl OnlineData {
    pub fn new(sender: js_sys::Function) -> OnlineData {
        let (channel_sender, channel_receiver) = channel(1000);
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

    pub async fn init(&mut self) {
        while let Some(msg) = self.receiver.next().await {
            let msg = GameMessage::from_bytes(&msg);
            match msg {
                GameMessage::MyID(id) => {
                    info!("My ID is: {}", id);
                    self.id = id;
                    return ();
                }
                _ => {}
            }
        }
    }

    pub fn ws_sender(&self) -> WSChannelSender {
        self.ws_sender.clone()
    }
}

impl OnlineData {
    pub fn send(&self, msg: GameMessage) {
        let msg = js_sys::Uint8Array::from(msg.to_bytes().as_slice());
        self.sender
            .call1(&JsValue::null(), &msg)
            .expect("should be possible to call");
    }

    pub fn tick(&mut self) {
        loop {
            let msg = self.receiver.try_next();
            let msg = match msg {
                Ok(Some(msg)) => msg,
                Err(_) => {
                    break;
                }
                _ => break,
            };
            let msg = GameMessage::from_bytes(&msg);
            match msg {
                GameMessage::ClientMessage(msg) => {
                    self.game_state.on_message(msg);
                }
                _ => {}
            }
        }
        self.game_state.tick(0.016);
    }
}

pub enum RunningMode {
    Local(
        game_server::GameServer,
        std::sync::mpsc::Receiver<MessageToSend>,
    ),
    Online(OnlineData),
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        match self {
            RunningMode::Local(game, _) => &game.game_state,
            RunningMode::Online(data) => &data.game_state,
        }
    }

    pub fn start_local() -> RunningMode {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut game = game_server::GameServer::new(sender);
        game.new_connection(0);
        info!("Local server started");
        return RunningMode::Local(game, receiver);
    }

    pub fn tick(&mut self) {
        match self {
            RunningMode::Local(game, receiver) => {
                game.tick(0.016);
                while let Ok(_) = receiver.try_recv() {
                    // draining
                }
            }
            RunningMode::Online(data) => {
                data.tick();
            }
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local(_, _) => 0,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn send_message(&mut self, msg: ClientMessage) {
        match self {
            RunningMode::Local(ref mut game, _) => {
                game.on_message(msg);
            }
            RunningMode::Online(data) => {
                let msg = GameMessage::ClientMessage(msg);
                info!("Sending message: {:?}", msg);
                data.send(msg);
            }
        }
    }
}
