use crate::{ClientMessage, ServerState};
use std::collections::HashMap;

type MessageToSend = (u64, String);

#[derive(Debug)]
pub enum GameMessage {
    NewConnection,
    ClientDisconnect(u64),
    ClientMessage(String),
    Tick,
}

pub enum GameServerMessageResult {
    PlayerID(u64),
    None,
}

pub struct GameServer {
    current_id: u64,
    game_state: ServerState,
    players: HashMap<u64, ()>,
    pub messages_to_send: Vec<MessageToSend>,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer {
            game_state: ServerState::new(),
            current_id: 0,
            players: HashMap::new(),
            messages_to_send: vec![],
        }
    }

    fn send_message_to_player(&mut self, id: u64, message: ClientMessage) {
        let msg = serde_json::to_string(&message).unwrap();
        self.messages_to_send.push((id, msg));
    }

    fn handle_create_player(&mut self) -> u64 {
        let id = self.current_id;
        self.players.insert(id, ());
        self.current_id += 1;
        id
    }

    fn broadcast_message(&mut self, message: ClientMessage) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, message.clone());
        }
    }

    pub fn on_message(&mut self, msg: GameMessage) -> anyhow::Result<GameServerMessageResult> {
        match msg {
            GameMessage::ClientMessage(msg) => {
                let msg = self.game_state.on_string_message(msg)?;
                self.broadcast_message(msg);
            }
            GameMessage::NewConnection {} => {
                let id = self.handle_create_player();
                let msg = ClientMessage::CreatePlayer { id };
                let my_id = ClientMessage::MarkMyID { id };
                self.send_message_to_player(id, my_id);
                let state = self.game_state.state_message();
                self.send_message_to_player(id, state);
                self.game_state.on_message(msg.clone());
                self.broadcast_message(msg);
                return Ok(GameServerMessageResult::PlayerID(id));
            }
            GameMessage::ClientDisconnect(id) => {
                self.players.remove(&id);
                let msg = ClientMessage::RemovePlayer { id };
                self.game_state.on_message(msg.clone());
                self.broadcast_message(msg);
            }
            GameMessage::Tick => {
                //self.game_state.on_message(ClientMessage::Tick);
            }
        }
        Ok(GameServerMessageResult::None)
    }
}
