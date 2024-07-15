use crate::{player::Player, ClientMessage, ServerState};
use futures::channel::mpsc::Sender;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

type PlayerSender = Sender<Vec<u8>>;

pub enum GameServerMessageResult {
    PlayerID(u64),
    None,
}

pub struct GameServer {
    pub game_state: ServerState,
    players: HashMap<u64, PlayerSender>,
    player_id_counter: u64,
    bots: Vec<Player>,
    rand_gen: fastrand::Rng,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer {
            game_state: ServerState::new(),
            players: HashMap::new(),
            player_id_counter: 0,
            bots: vec![],
            rand_gen: fastrand::Rng::new(),
        }
    }

    fn add_bot(&mut self) {
        let bot = Player::new(self.next_player_id());
        self.bots.push(bot);
    }

    pub fn next_player_id(&mut self) -> u64 {
        self.player_id_counter += 1;
        self.player_id_counter
    }

    fn send_message_to_player(&mut self, id: u64, message: GameMessage) {
        if let Some(sender) = self.players.get_mut(&id) {
            sender
                .try_send(message.to_bytes())
                .expect("Failed to send message to player");
        }
    }

    fn broadcast_message(&mut self, message: ClientMessage) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, GameMessage::ClientMessage(message.clone()));
        }
    }

    pub fn on_message(&mut self, msg: Vec<u8>) {
        let msg = GameMessage::from_bytes(&msg);
        match msg {
            GameMessage::ClientMessage(msg) => self.game_state.on_message(msg),
            _ => {}
        }
    }

    pub fn new_connection(&mut self, sender: PlayerSender) -> u64 {
        let id = self.next_player_id();
        self.players.insert(id, sender);
        let msg = ClientMessage::CreatePlayer { id };
        let state = self.game_state.state_message();
        self.send_message_to_player(id, GameMessage::ClientMessage(state));
        self.game_state.on_message(msg.clone());
        let my_id = GameMessage::MyID(id);
        self.send_message_to_player(id, my_id);
        self.add_bot();
        return id;
    }

    pub fn disconnect_player(&mut self, id: u64) {
        self.players.remove(&id);
        let msg = ClientMessage::RemovePlayer { id };
        self.game_state.on_message(msg.clone());
    }

    fn handle_bots(&mut self) {
        self.bots.iter_mut().for_each(|bot| {
            bot.tick(&self.game_state);
            if !bot.has_ships(&self.game_state) {
                for _ in 0..10 {
                    let x = self.rand_gen.f64() * 10.0;
                    let y = self.rand_gen.f64() * 10.0;
                    if self.game_state.game_map.is_allowed_place(x, y) {
                        bot.create_ship(x, y)
                    }
                }
            }
            while let Some(msg) = bot.next_message() {
                self.game_state.on_message(msg);
            }
        });
    }

    pub fn tick(&mut self, dt: f64) {
        self.handle_bots();
        self.game_state.tick(dt);
        self.broadcast_message(self.game_state.state_message());
    }
}
