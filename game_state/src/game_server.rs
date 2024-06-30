use crate::{ClientMessage, ServerState};
use std::collections::HashMap;

const DT: f64 = 0.016;

type MessageToSend = (u64, String);

#[derive(Debug)]
pub enum GameMessage {
    NewConnection(u64),
    ClientDisconnect(u64),
    ClientMessage(String),
    Tick,
}

pub enum GameServerMessageResult {
    PlayerID(u64),
    None,
}

pub struct GameServer {
    pub game_state: ServerState,
    players: HashMap<u64, ()>,
    player_id_counter: u64,
    pub messages_to_send: Vec<MessageToSend>,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer {
            game_state: ServerState::new(),
            players: HashMap::new(),
            messages_to_send: vec![],
            player_id_counter: 0,
        }
    }

    pub fn next_player_id(&mut self) -> u64 {
        self.player_id_counter += 1;
        self.player_id_counter
    }

    fn send_message_to_player(&mut self, id: u64, message: ClientMessage) {
        let msg = serde_json::to_string(&message).unwrap();
        self.messages_to_send.push((id, msg));
    }

    fn handle_create_player(&mut self, id: u64) {
        self.players.insert(id, ());
    }

    fn broadcast_message(&mut self, message: ClientMessage) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, message.clone());
        }
    }

    pub fn on_string_message(&mut self, msg: String) -> anyhow::Result<ClientMessage> {
        let msg: ClientMessage = serde_json::from_str(&msg)?;
        Ok(msg)
    }

    pub fn on_message(&mut self, msg: GameMessage) -> anyhow::Result<GameServerMessageResult> {
        match msg {
            GameMessage::ClientMessage(msg) => {
                let msg = self.game_state.on_string_message(msg)?;
                self.broadcast_message(msg);
            }
            GameMessage::NewConnection(id) => {
                self.handle_create_player(id);
                let msg = ClientMessage::CreatePlayer { id };
                let state = self.game_state.state_message();
                self.send_message_to_player(id, state);
                self.game_state.on_message(msg.clone());
                self.broadcast_message(msg);
            }
            GameMessage::ClientDisconnect(id) => {
                self.players.remove(&id);
                let msg = ClientMessage::RemovePlayer { id };
                self.game_state.on_message(msg.clone());
                self.broadcast_message(msg);
            }
            GameMessage::Tick => self.tick(),
        }
        Ok(GameServerMessageResult::None)
    }

    fn tick(&mut self) {
        self.game_state.evolve_ships(DT)
    }
}
