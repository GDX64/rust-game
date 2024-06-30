use crate::{game_server, ClientMessage, GameMessage};
use log::{error, info};

pub enum RunningMode {
    Local(game_server::GameServer),
    None,
}

impl RunningMode {
    pub fn poll_messages(&mut self) -> Vec<(u64, String)> {
        match self {
            RunningMode::Local(game) => std::mem::replace(&mut game.messages_to_send, Vec::new()),
            RunningMode::None => {
                vec![]
            }
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
            }
            RunningMode::None => {}
        };
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
            RunningMode::None => {}
        }
    }
}
