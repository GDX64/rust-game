use crate::{game_server, ClientMessage, GameMessage, ServerState};
use log::info;
use wasm_bindgen::JsValue;

pub struct OnlineData {
    sender: js_sys::Function,
    game_state: ServerState,
    id: u64,
}

impl OnlineData {
    pub fn new(sender: js_sys::Function) -> OnlineData {
        OnlineData {
            sender,
            game_state: ServerState::new(),
            id: 0,
        }
    }

    pub fn send(&self, msg: GameMessage) {
        let msg = JsValue::from_str(&msg.to_string());
        self.sender
            .call1(&JsValue::null(), &msg)
            .expect("should be possible to call");
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
                game.tick();
                game.messages_to_send.drain(..);
            }
            RunningMode::None(_) => {}
            RunningMode::Online(_) => {}
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local(_) => 0,
            RunningMode::None(_) => 0,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn on_message(&mut self, msg: String) {
        let msg: GameMessage =
            serde_json::from_str(&msg).expect("should be possible to deserialize");
        match self {
            RunningMode::Local(_) => {}
            RunningMode::None(_) => {}
            RunningMode::Online(data) => {
                match msg {
                    GameMessage::MyID(id) => {
                        data.id = id;
                    }
                    GameMessage::ClientMessage(msg) => {
                        data.game_state.on_message(msg);
                    }
                    _ => {}
                }
            }
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
                data.send(msg);
            }
        }
    }
}
