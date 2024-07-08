use serde::{Deserialize, Serialize};

use crate::{ClientMessage, ServerState};
use std::{collections::HashMap, sync::mpsc::Sender};

pub type MessageToSend = (u64, GameMessage);

#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {
    ClientMessage(ClientMessage),
    MyID(u64),
    None,
}

impl GameMessage {
    pub fn from_string(msg: String) -> GameMessage {
        let msg: GameMessage = serde_json::from_str(&msg).unwrap_or(GameMessage::None);
        return msg;
    }

    pub fn to_string(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("error serializing message: {:?}", e);
                "error".to_string()
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> GameMessage {
        bincode::deserialize(bytes).unwrap_or(GameMessage::None)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize")
    }
}

impl From<&[u8]> for GameMessage {
    fn from(bytes: &[u8]) -> GameMessage {
        GameMessage::from_bytes(bytes)
    }
}

impl From<String> for GameMessage {
    fn from(msg: String) -> GameMessage {
        GameMessage::from_string(msg)
    }
}

pub enum GameServerMessageResult {
    PlayerID(u64),
    None,
}

pub struct GameServer {
    pub game_state: ServerState,
    players: HashMap<u64, ()>,
    player_id_counter: u64,
    sender: Sender<MessageToSend>,
}

impl GameServer {
    pub fn new(sender: Sender<MessageToSend>) -> GameServer {
        GameServer {
            game_state: ServerState::new(),
            players: HashMap::new(),
            sender,
            player_id_counter: 0,
        }
    }

    pub fn next_player_id(&mut self) -> u64 {
        self.player_id_counter += 1;
        self.player_id_counter
    }

    fn send_message_to_player(&mut self, id: u64, message: GameMessage) {
        if let Err(error) = self.sender.send((id, message)) {
            eprintln!("error sending message: {:?}", error);
        }
    }

    fn handle_create_player(&mut self, id: u64) {
        self.players.insert(id, ());
    }

    fn broadcast_message(&mut self, message: ClientMessage) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, GameMessage::ClientMessage(message.clone()));
        }
    }

    pub fn on_message(&mut self, msg: ClientMessage) {
        self.game_state.on_message(msg.clone());
    }

    pub fn new_connection(&mut self, id: u64) {
        self.handle_create_player(id);
        let msg = ClientMessage::CreatePlayer { id };
        let state = self.game_state.state_message();
        self.send_message_to_player(id, GameMessage::ClientMessage(state));
        self.game_state.on_message(msg.clone());
        let my_id = GameMessage::MyID(id);
        self.send_message_to_player(id, my_id);
    }

    pub fn disconnect_player(&mut self, id: u64) {
        self.players.remove(&id);
        let msg = ClientMessage::RemovePlayer { id };
        self.game_state.on_message(msg.clone());
    }

    pub fn tick(&mut self, dt: f64) {
        self.game_state.tick(dt);
        self.broadcast_message(self.game_state.state_message());
    }
}
