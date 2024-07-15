use crate::player::Player;
use crate::ws_channel::WSChannel;
use crate::{game_server, ClientMessage, GameMessage, ServerState};
use futures::channel::mpsc::{channel, Receiver};
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
    Local {
        game: game_server::GameServer,
        receiver: Receiver<Vec<u8>>,
        state: ServerState,
        id: u64,
    },
    Online(OnlineData),
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        match self {
            RunningMode::Local { ref state, .. } => state,
            RunningMode::Online(data) => &data.game_state,
        }
    }

    pub fn start_local() -> RunningMode {
        let (sender, receiver) = channel(100);
        let mut game = game_server::GameServer::new();
        let player_id = game.new_connection(sender);
        info!("Local server started");
        return RunningMode::Local {
            game,
            receiver,
            id: player_id,
            state: ServerState::new(),
        };
    }

    pub fn tick(&mut self) {
        match self {
            RunningMode::Local {
                receiver,
                state,
                game,
                ..
            } => {
                game.tick(0.016);
                while let Ok(Some(msg)) = receiver.try_next() {
                    let game_message = GameMessage::from_bytes(&msg);
                    match game_message {
                        GameMessage::ClientMessage(msg) => {
                            state.on_message(msg);
                        }
                        _ => {}
                    }
                }
                state.tick(0.016);
            }
            RunningMode::Online(data) => {
                data.tick();
            }
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local { id, .. } => *id,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn send_message(&mut self, msg: ClientMessage) {
        match self {
            RunningMode::Local { ref mut game, .. } => {
                let msg = GameMessage::ClientMessage(msg);
                game.on_message(msg.to_bytes());
            }
            RunningMode::Online(data) => {
                let msg = GameMessage::ClientMessage(msg);
                data.send(msg);
            }
        }
    }
}
