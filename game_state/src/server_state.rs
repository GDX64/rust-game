use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerState {
    name: String,
    position: (f64, f64),
    id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadCastState {
    players: Vec<PlayerState>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMessage {
    SetPlayerName { name: String, id: u64 },
    PlayerCreatedResponse { id: u64 },
    MovePlayer { position: (f64, f64), id: u64 },
    BroadCastState { state: BroadCastState },
}

pub struct ServerState {
    players: HashMap<u64, PlayerState>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    fn handle_move_player(&mut self, id: u64, position: (f64, f64)) {
        if let Some(player) = self.players.get_mut(&id) {
            player.position = position;
        }
    }

    fn handle_set_player_name(&mut self, name: String, id: u64) {
        if let Some(player) = self.players.get_mut(&id) {
            player.name = name;
        }
    }

    pub fn on_string_message(&mut self, msg: String) -> anyhow::Result<ClientMessage> {
        let msg: ClientMessage = serde_json::from_str(&msg)?;
        self.on_message(msg.clone());
        Ok(msg)
    }

    fn on_message(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::SetPlayerName { name, id } => {
                self.handle_set_player_name(name, id);
            }
            ClientMessage::MovePlayer { position, id } => {
                self.handle_move_player(id, position);
            }
            _ => {}
        }
    }
}
