use crate::{player::Player, ClientMessage, ServerState};
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {
    ClientMessage(ClientMessage),
    AddBot,
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
        self.game_state
            .on_message(ClientMessage::CreatePlayer { id: bot.id });
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
            GameMessage::AddBot => self.add_bot(),
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
        return id;
    }

    pub fn disconnect_player(&mut self, id: u64) {
        self.players.remove(&id);
        let msg = ClientMessage::RemovePlayer { id };
        self.game_state.on_message(msg);
    }

    fn handle_bots(&mut self) {
        self.bots.iter_mut().for_each(|bot| {
            bot.tick(&self.game_state);

            let mut enemies = self
                .game_state
                .ship_collection
                .values()
                .filter(|ship| ship.player_id != bot.id);

            bot.player_ships(&self.game_state).for_each(|ship| {
                let cannon = ship.find_available_cannon(self.game_state.current_time);
                if cannon.is_none() {
                    return;
                };
                if let Some(enemy) = enemies.next() {
                    bot.shoot_at(ship.id, enemy.position.0, enemy.position.1);
                };
            });

            if bot.number_of_ships(&self.game_state) < 5 {
                for _ in 0..10 {
                    let x = self.rand_gen.f64() * 1000.0 - 500.0;
                    let y = self.rand_gen.f64() * 1000.0 - 500.0;
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
