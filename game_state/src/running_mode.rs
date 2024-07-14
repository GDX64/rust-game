use crate::ws_channel::WSChannel;
use crate::{game_server, ClientMessage, GameMessage, MessageToSend, ServerState};
use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineData {
    ws: WSChannel,
    game_state: ServerState,
    id: u64,
}

#[wasm_bindgen]
impl OnlineData {
    pub fn new(url: &str) -> OnlineData {
        OnlineData {
            ws: WSChannel::new(url),
            game_state: ServerState::new(),
            id: 0,
        }
    }

    pub async fn init(&mut self) {
        while let Some(msg) = self.ws.next().await {
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
}

impl OnlineData {
    pub fn send(&mut self, msg: GameMessage) {
        self.ws.send(msg.to_bytes());
    }

    pub fn tick(&mut self) {
        loop {
            let msg = self.ws.receive();
            let msg = match msg {
                Some(msg) => msg,
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
