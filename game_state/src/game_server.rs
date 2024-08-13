use crate::{player::Player, ServerState, ShipState, StateMessage};
use futures::channel::mpsc::Sender;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_BOTS: usize = 5;
const SYNC_EVERY_N_FRAMES: u64 = 1000;
pub const TICK_TIME: f64 = 1.0 / 60.0;

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

    pub fn serialize_arr(arr: &Vec<GameMessage>) -> Vec<u8> {
        bincode::serialize(arr).expect("Failed to serialize")
    }

    pub fn from_arr_bytes(bytes: &[u8]) -> Vec<GameMessage> {
        bincode::deserialize(bytes).unwrap_or(vec![])
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
}

type PlayerSender = Sender<Vec<u8>>;
struct PlayerBufferSenderPair {
    buffer: Vec<GameMessage>,
    sender: PlayerSender,
}

pub enum GameServerMessageResult {
    PlayerID(u64),
    None,
}

pub struct GameServer {
    pub game_state: ServerState,
    players: HashMap<u64, PlayerBufferSenderPair>,
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
            rand_gen: fastrand::Rng::with_seed(0),
            frames: 0,
            frame_inputs: vec![],
        }
    }

    fn add_bot(&mut self) {
        if self.bots.len() > MAX_BOTS {
            return;
        }
        let bot = Player::new(self.next_player_id());
        self.add_to_frame(StateMessage::CreatePlayer { id: bot.id });
        self.bots.push(bot);
    }

    fn remove_bot(&mut self) {
        if let Some(bot) = self.bots.pop() {
            self.add_to_frame(StateMessage::RemovePlayer { id: bot.id });
        }
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
        };
    }

    pub fn new_connection(&mut self, sender: PlayerSender) -> u64 {
        let id = self.next_player_id();
        let pair = PlayerBufferSenderPair {
            buffer: vec![],
            sender,
        };
        self.players.insert(id, pair);

        let create_player_msg = StateMessage::CreatePlayer { id };
        self.add_to_frame(create_player_msg.clone());

        let my_id = GameMessage::MyID(id);
        self.send_message_to_player(id, my_id);

        let state = self.game_state.state_message();
        self.send_message_to_player(id, GameMessage::FrameMessage(vec![state]));

        let origin = (-200.0, 0.0);

        for j in 0..10 {
            for i in 0..10 {
                let x = (i * 20) as f64 + origin.0;
                let y = j as f64 * 20.0 + origin.1;
                if self.game_state.game_map.is_allowed_place(x, y) {
                    let mut ship = ShipState::default();
                    ship.position = (x, y);
                    ship.player_id = id;
                    self.add_to_frame(StateMessage::CreateShip { ship });
                }
            }
        }

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
        self.add_to_frame(msg);
    }

    fn handle_bots(&mut self) {
        self.bots.iter_mut().for_each(|bot| {
            bot.tick(&self.game_state);
            bot.select_all(&self.game_state);
            bot.auto_shoot(&self.game_state);
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
            self.add_to_frame(msg);
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

    fn flush_send_buffers(&mut self) {
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
            self.disconnect_player(id);
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
