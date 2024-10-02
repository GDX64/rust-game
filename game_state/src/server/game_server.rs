use crate::{
    bot_player::BotPlayer,
    ship::ShipState,
    utils::vectors::V2D,
    wasm_game::{ServerState, StateMessage, PLAYER_START_SHIPS},
};
use futures::channel::mpsc::Sender;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_BOTS: usize = 10;
const SYNC_EVERY_N_FRAMES: u64 = 1000;
pub const TICK_TIME: f64 = 1.0 / 60.0;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameMessage {
    FrameMessage(Vec<StateMessage>),
    InputMessage(StateMessage),
    AddBot,
    AddBotShipAt(f64, f64),
    RemoveBot,
    PlayerCreated { x: f64, y: f64, id: u64 },
    AskBroadcast { player: u64 },
    ConnectionDown,
    Reconnection,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StateEvent {
    MyID(u64),
    PositionReset(V2D),
}

impl GameMessage {
    pub fn serialize_arr(arr: &Vec<GameMessage>) -> Vec<u8> {
        bincode::serialize(arr).expect("Failed to serialize")
    }

    pub fn from_arr_bytes(bytes: &[u8]) -> Vec<GameMessage> {
        bincode::deserialize(bytes).unwrap_or(vec![])
    }
}

type PlayerSender = Sender<Vec<u8>>;
struct PlayerBufferSenderPair {
    buffer: Vec<GameMessage>,
    sender: PlayerSender,
    connection_down_time: Option<u64>,
}

pub struct GameServer {
    pub game_state: ServerState,
    players: HashMap<u64, PlayerBufferSenderPair>,
    player_id_counter: u64,
    bots: Vec<BotPlayer>,
    frame_inputs: Vec<StateMessage>,
    rng: fastrand::Rng,
    frames: u64,
    pub name: String,
}

impl GameServer {
    pub fn new() -> GameServer {
        GameServer {
            game_state: ServerState::new(),
            players: HashMap::new(),
            player_id_counter: 0,
            bots: vec![],
            rng: fastrand::Rng::with_seed(1),
            frames: 0,
            frame_inputs: vec![],
            name: "default".to_string(),
        }
    }

    pub fn get_player_count(&self) -> usize {
        self.players.len() + self.bots.len()
    }

    fn add_bot(&mut self) {
        if self.bots.len() > MAX_BOTS {
            return;
        }
        let mut bot = BotPlayer::new(self.next_player_id());
        let max_size = self.game_state.game_map.dim;
        let x = (self.rng.f64() - 0.5) * max_size / 2.0;
        let y = (self.rng.f64() - 0.5) * max_size / 2.0;
        for _ in 0..PLAYER_START_SHIPS {
            bot.player.create_ship(x, y)
        }
        let name = format!("Bot {}", bot.player.id);
        self.add_to_frame(StateMessage::CreatePlayer {
            id: bot.player.id,
            name,
        });
        self.bots.push(bot);
    }

    fn remove_bot(&mut self, id: u64) {
        if let Some(bot) = self.bots.iter().find(|bot| bot.player.id == id) {
            self.add_to_frame(StateMessage::RemovePlayer { id: bot.player.id });
        }
        self.bots.retain(|bot| bot.player.id != id);
    }

    pub fn next_player_id(&mut self) -> u64 {
        self.player_id_counter += 1;
        self.player_id_counter
    }

    fn send_message_to_player(&mut self, id: u64, message: GameMessage) {
        if let Some(sender) = self.players.get_mut(&id) {
            sender.buffer.push(message);
        }
    }

    fn broadcast(&mut self, message: GameMessage) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, message.clone());
        }
    }

    pub fn on_message(&mut self, msg: Vec<u8>) {
        let msg = GameMessage::from_arr_bytes(&msg);
        for msg in msg {
            self.handle_single_message(msg);
        }
    }

    fn handle_single_message(&mut self, msg: GameMessage) {
        match msg {
            GameMessage::FrameMessage(_msg) => {
                log::error!("Server should not receive FrameMessage");
            }
            GameMessage::InputMessage(msg) => self.add_to_frame(msg),
            GameMessage::AddBot => self.add_bot(),
            GameMessage::RemoveBot => {
                if let Some(bot) = self.bots.first() {
                    self.remove_bot(bot.player.id);
                }
            }
            GameMessage::AddBotShipAt(x, y) => {
                if let Some(bot) = self.bots.last_mut() {
                    bot.player.create_ship(x, y);
                } else {
                    self.add_bot();
                }
            }
            GameMessage::AskBroadcast { player } => {
                let state = self.game_state.state_message();
                self.send_message_to_player(player, GameMessage::FrameMessage(vec![state]));
            }
            // Those messages should not be received in the server
            GameMessage::PlayerCreated { .. } => {}
            GameMessage::None => {}
            GameMessage::ConnectionDown => {}
            GameMessage::Reconnection => {}
        };
    }

    pub fn new_connection(&mut self, sender: PlayerSender, id: Option<u64>) -> u64 {
        if let Some(id) = id {
            if let Some(player) = self.players.get_mut(&id) {
                player.sender = sender;
                player.connection_down_time = None;
                return id;
            }
            log::error!("Player {} not found", id);
        }

        let id = self.next_player_id();
        let pair = PlayerBufferSenderPair {
            buffer: vec![],
            sender,
            connection_down_time: None,
        };

        let has_no_players = self.players.is_empty();

        self.players.insert(id, pair);

        let name = format!("Player {}", id);
        let create_player_msg = StateMessage::CreatePlayer { id, name };
        self.add_to_frame(create_player_msg.clone());

        let map_size = self.game_state.game_map.dim * 0.8;
        let start_x = (self.rng.f64() - 0.5) * map_size;
        let start_y = (self.rng.f64() - 0.5) * map_size;

        self.send_message_to_player(
            id,
            GameMessage::PlayerCreated {
                x: start_x,
                y: start_y,
                id,
            },
        );

        self.send_message_to_player(id, GameMessage::Reconnection);

        for _ in 0..PLAYER_START_SHIPS {
            let mut ship = ShipState::default();
            ship.position.x = start_x;
            ship.position.y = start_y;
            ship.player_id = id;
            self.add_to_frame(StateMessage::CreateShip { ship });
        }

        if has_no_players {
            for _ in 0..MAX_BOTS {
                self.add_bot();
            }
        }

        return id;
    }

    pub fn on_player_connection_down(&mut self, id: u64) {
        info!(
            "Player {} connection down, total players {}",
            id,
            self.players.len()
        );
        if let Some(player) = self.players.get_mut(&id) {
            player.connection_down_time = Some(crate::utils::system_things::get_time());
        }
    }

    fn handle_bots(&mut self) {
        self.bots.iter_mut().for_each(|bot| {
            bot.tick(TICK_TIME, &self.game_state);
        });
        let bot_messages = self
            .bots
            .iter()
            .flat_map(|bot| bot.player.collect_messages())
            .collect::<Vec<_>>();
        for msg in bot_messages {
            self.add_to_frame(msg);
        }
        let dead_bots: Vec<_> = self
            .bots
            .iter()
            .filter(|bot| bot.is_dead())
            .map(|bot| bot.player.id)
            .collect();
        for bot in dead_bots {
            self.remove_bot(bot);
            self.add_bot();
        }
    }

    pub fn tick(&mut self, time: f64) {
        if self.players.is_empty() {
            // no players, no need to tick
            return;
        }
        self.frames += 1;
        self.handle_bots();

        self.add_to_frame(StateMessage::Tick(time));
        self.run_inputs();

        if self.frames % SYNC_EVERY_N_FRAMES == 0 {
            self.frame_inputs = vec![self.game_state.state_message()];
        }
        self.flush_frame_inputs();
        self.flush_send_buffers();
    }

    pub fn flush_send_buffers(&mut self) {
        let player_ids: Vec<u64> = self.players.keys().cloned().collect();
        let mut player_errors = vec![];
        for id in player_ids {
            if let Some(player) = self.players.get_mut(&id) {
                let messages = GameMessage::serialize_arr(&player.buffer);
                match player.sender.try_send(messages) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error sending message to player {}: {:?}", id, e);
                        log::info!("Player {} will be removed", id);
                        player_errors.push(id);
                    }
                }
                player.buffer.clear();
            }
        }
        for id in player_errors {
            self.on_player_connection_down(id);
        }
    }

    fn flush_frame_inputs(&mut self) {
        if self.frame_inputs.is_empty() {
            return;
        }
        let frame_message = GameMessage::FrameMessage(self.frame_inputs.clone());
        self.broadcast(frame_message);
        self.frame_inputs.clear();
    }

    fn run_inputs(&mut self) {
        for msg in self.frame_inputs.iter() {
            self.game_state.on_message(msg.clone());
        }
    }

    fn add_to_frame(&mut self, msg: StateMessage) {
        self.frame_inputs.push(msg);
    }
}