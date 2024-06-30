use crate::{game_server, ClientMessage, GameMessage, ServerState};
use log::{error, info};

pub enum RunningMode {
    Local(game_server::GameServer),
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
        }
    }

    pub fn start_local() -> RunningMode {
        let player_message = GameMessage::NewConnection(0);
        let mut game = game_server::GameServer::new();
        game.on_message(player_message)
            .expect("should be possible to start local server");

        info!("Local server started");
        return RunningMode::Local(game);
    }

    pub fn tick(&mut self) {
        match self {
            RunningMode::Local(game) => {
                game.on_message(GameMessage::Tick)
                    .expect("should be possible to tick");
                game.messages_to_send.drain(..);
            }
            RunningMode::None(_) => {}
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local(_) => 0,
            RunningMode::None(_) => 0,
        }
    }

    pub fn send_message(&mut self, msg: ClientMessage) {
        match self {
            RunningMode::Local(ref mut game) => {
                let msg = serde_json::to_string(&msg).expect("should be possible to serialize");
                let msg = GameMessage::ClientMessage(msg);
                match game.on_message(msg) {
                    Err(err) => error!("Error sending message: {:?}", err),
                    Ok(_) => {}
                }
            }
            RunningMode::None(_) => {}
        }
    }
}
