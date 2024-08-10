use crate::{player::Player, ServerState, StateMessage};
use futures::channel::mpsc::Sender;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_BOTS: usize = 5;
const SYNC_EVERY_N_FRAMES: u64 = 1000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameMessage {
    FrameMessage(Vec<StateMessage>),
    InputMessage(StateMessage),
    AddBot,
    AddBotShipAt(f64, f64),
    RemoveBot,
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
    frame_inputs: Vec<StateMessage>,
    rand_gen: fastrand::Rng,
    frames: u64,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer {
            game_state: ServerState::new(),
            players: HashMap::new(),
            player_id_counter: 0,
            bots: vec![],
            rand_gen: fastrand::Rng::new(),
            frames: 0,
            frame_inputs: vec![],
        }
    }

    fn add_bot(&mut self) {
        if self.bots.len() > MAX_BOTS {
            return;
        }
        let bot = Player::new(self.next_player_id());
        self.input_message(StateMessage::CreatePlayer { id: bot.id });
        self.bots.push(bot);
    }

    fn remove_bot(&mut self) {
        if let Some(bot) = self.bots.pop() {
            self.input_message(StateMessage::RemovePlayer { id: bot.id });
        }
    }

    pub fn next_player_id(&mut self) -> u64 {
        self.player_id_counter += 1;
        self.player_id_counter
    }

    fn send_message_to_player(&mut self, id: u64, message: GameMessage) {
        if let Some(sender) = self.players.get_mut(&id) {
            match sender.try_send(message.to_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error sending message to player {}: {:?}", id, e);
                    log::info!("Player {} will be removed", id);
                    self.disconnect_player(id);
                }
            }
        }
    }

    fn broadcast(&mut self, message: GameMessage) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, message.clone());
        }
    }

    pub fn on_message(&mut self, msg: Vec<u8>) {
        let msg = GameMessage::from_bytes(&msg);
        match msg {
            GameMessage::FrameMessage(_msg) => {
                log::error!("Server should not receive FrameMessage");
            }
            GameMessage::InputMessage(msg) => self.input_message(msg),
            GameMessage::AddBot => self.add_bot(),
            GameMessage::RemoveBot => self.remove_bot(),
            GameMessage::AddBotShipAt(x, y) => {
                if let Some(bot) = self.bots.last_mut() {
                    bot.create_ship(x, y);
                } else {
                    self.add_bot();
                }
            }
            GameMessage::MyID(_) => {}
            GameMessage::None => {}
        }
    }

    pub fn new_connection(&mut self, sender: PlayerSender) -> u64 {
        let id = self.next_player_id();
        self.players.insert(id, sender);

        let create_player_msg = StateMessage::CreatePlayer { id };
        self.input_message(create_player_msg.clone());

        let my_id = GameMessage::MyID(id);
        self.send_message_to_player(id, my_id);

        let state = self.game_state.state_message();
        self.send_message_to_player(id, GameMessage::FrameMessage(vec![state]));

        return id;
    }

    pub fn disconnect_player(&mut self, id: u64) {
        self.players.remove(&id);
        info!(
            "Player {} disconnected, total players {}",
            id,
            self.players.len()
        );
        let msg = StateMessage::RemovePlayer { id };
        self.input_message(msg);
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
                if ship.last_shoot_time + 1.0 > self.game_state.current_time {
                    return;
                }
                let cannon = ship.find_available_cannon(self.game_state.current_time);

                if cannon.is_none() {
                    return;
                };
                if let Some(enemy) = enemies.next() {
                    bot.shoot_at_with(ship.id, enemy.position.0, enemy.position.1);
                };
            });

            if bot.number_of_ships(&self.game_state) < 5 {
                for _ in 0..5 {
                    let x = self.rand_gen.f64() * 1000.0 - 500.0;
                    let y = self.rand_gen.f64() * 1000.0 - 500.0;
                    if self.game_state.game_map.is_allowed_place(x, y) {
                        bot.create_ship(x, y)
                    }
                }
            }
        });
        let bot_messages = self
            .bots
            .iter()
            .flat_map(|bot| bot.collect_messages())
            .collect::<Vec<_>>();
        for msg in bot_messages {
            self.input_message(msg);
        }
    }

    pub fn tick(&mut self, time: f64) {
        if self.players.is_empty() {
            // no players, no need to tick
            return;
        }
        self.frames += 1;
        self.handle_bots();
        self.input_message(StateMessage::Tick(time));
        if self.frames % SYNC_EVERY_N_FRAMES == 0 {
            self.frame_inputs = vec![self.game_state.state_message()];
        }
        self.flush_frame_inputs();
    }

    fn flush_frame_inputs(&mut self) {
        if self.frame_inputs.is_empty() {
            return;
        }
        let frame_message = GameMessage::FrameMessage(self.frame_inputs.clone());
        self.broadcast(frame_message);
        self.frame_inputs.clear();
    }

    fn input_message(&mut self, msg: StateMessage) {
        self.game_state.on_message(msg.clone());
        self.frame_inputs.push(msg);
    }
}
