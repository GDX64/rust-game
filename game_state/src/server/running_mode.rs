use super::game_server::GameMessage;
use crate::player::Player;
use crate::player_state::PlayerID;
use crate::server::game_server::ConnectionID;
use crate::server::Client;
use crate::server_state::{ServerState, StateMessage};
use crate::utils::event_hub::{EventHub, EventKey, Subscription};
use crate::utils::vectors::V2D;
use crate::{BotPlayer, WrappedActor};
use futures::StreamExt;
use log::info;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RunningEvent {
    PlayerCreated { id: PlayerID, x: f64, y: f64 },
    PositionChanged(V2D),
    Pong,
    Connected,
    BotDead(PlayerID),
    StatePong { id: u64 },
    Tick { tick: u64 },
}

impl EventKey for RunningEvent {}

pub struct RunningMode {
    game_state: ServerState,
    client: Box<dyn Client>,
    connection_id: Option<ConnectionID>,
    players: BTreeMap<PlayerID, Player>,
    bots: BTreeMap<PlayerID, BotPlayer>,
    pub start_position: V2D,
    pub events: EventHub<RunningEvent>,
    rng: fastrand::Rng,
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        &self.game_state
    }

    pub fn wrapped(mut client: Box<dyn Client>) -> WrappedActor<RunningMode> {
        let mut receiver = client.take_receiver().expect("Failed to take receiver");
        let rn = RunningMode {
            game_state: ServerState::new(0, 1),
            client,
            connection_id: None,
            players: BTreeMap::new(),
            bots: BTreeMap::new(),
            start_position: V2D::new(0.0, 0.0),
            events: EventHub::new(),
            rng: fastrand::Rng::with_seed(0),
        };
        let wr = WrappedActor::new(rn);
        let mut wr_clone = wr.clone();

        //possible leak here
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                wr_clone
                    .with_state(|inner| {
                        inner.on_game_message(msg);
                    })
                    .await;
            }
        });
        return wr;
    }

    fn tick(&mut self, dt: f64) {
        self.client.tick(dt);
        let state = &self.game_state;
        let mut messages_to_send = vec![];
        let mut bots_to_remove = vec![];
        for bot in self.bots.values_mut() {
            bot.tick(dt, state);
            bot.player.collect_messages().into_iter().for_each(|msg| {
                messages_to_send.push(msg);
            });
            if bot.is_dead() {
                log::info!("bot is dead");
                bots_to_remove.push(bot.player.id);
                self.events.notify(RunningEvent::BotDead(bot.player.id));
            }
        }
        for id in bots_to_remove {
            self.bots.remove(&id);
        }
        for msg in messages_to_send {
            self.send_game_message(GameMessage::InputMessage(msg));
        }
    }

    fn on_game_message(&mut self, msg: GameMessage) {
        match msg {
            GameMessage::FrameMessage(msg) => {
                msg.into_iter().for_each(|msg| {
                    if let StateMessage::Ping { id } = &msg {
                        self.events.notify(RunningEvent::StatePong { id: *id });
                    }
                    if let StateMessage::Tick { tick, dt } = &msg {
                        self.events.notify(RunningEvent::Tick { tick: *tick });
                        self.tick(*dt);
                    }
                    self.game_state.on_message(msg);
                });
            }
            GameMessage::PlayerCreated { id, x, y, bot } => {
                info!("Player Created with id: {:?}", id);
                self.events.notify(RunningEvent::PlayerCreated { id, x, y });
                self.events
                    .notify(RunningEvent::PositionChanged(self.start_position));
                if bot {
                    self.create_bot(id, x, y);
                } else {
                    self.create_player(id, x, y);
                }
            }
            GameMessage::Reconnection { id, seed } => {
                self.events.notify(RunningEvent::Connected);
                self.game_state = ServerState::new(seed, 1);
                self.connection_id = Some(id);
                self.send_game_message(GameMessage::AskBroadcast { connection_id: id });
            }
            GameMessage::ConnectionDown => {
                self.client.reconnect();
            }
            GameMessage::Pong => {
                self.events.notify(RunningEvent::Pong);
            }
            _ => {}
        }
    }

    fn create_bot(&mut self, player_id: PlayerID, x: f64, y: f64) {
        let mut bot = BotPlayer::new(player_id);
        bot.player.position = V2D::new(x, y);
        self.bots.insert(player_id, bot);
    }

    fn create_player(&mut self, player_id: PlayerID, x: f64, y: f64) {
        let mut player = Player::new(player_id);
        player.position = V2D::new(x, y);
        self.players.insert(player_id, player);
    }

    pub fn ask_ping(&mut self) -> u64 {
        let id = self.rng.u64(0..u64::MAX);
        self.send_game_message(GameMessage::InputMessage(StateMessage::Ping { id }));
        return id;
    }

    pub fn ask_create_player(&mut self, bot: bool) {
        log::info!("Asking server to create player, bot={}", bot);
        let Some(connection_id) = self.connection_id else {
            log::error!("No connection ID set, cannot create player");
            return;
        };
        let name = get_fake_name(&mut self.rng);
        self.send_game_message(GameMessage::CreatePlayer {
            name: Some(name),
            flag: None,
            bot,
            connection_id,
        });
    }

    pub fn clear_flags(&mut self) {
        self.game_state.clear_flags();
    }

    pub fn id(&self) -> PlayerID {
        self.players.keys().next().cloned().unwrap_or_default()
    }

    pub fn send_game_message(&mut self, msg: GameMessage) {
        self.client.send(msg);
    }
}

#[derive(Debug)]
pub enum RunningModeMessage {
    Tick(f64),
    GameMessage(GameMessage),
    CreateBot,
}

impl WrappedActor<RunningMode> {
    pub async fn subscribe(&mut self) -> Subscription<RunningEvent> {
        let sub = self
            .with_state(|state| {
                return state.events.subscribe(1000);
            })
            .await;
        return sub;
    }
}

fn get_fake_name(rng: &mut fastrand::Rng) -> String {
    let first = rng.u32(0..MOCK_NAME_SET.len() as u32);
    let last = rng.u32(0..MOCK_LAST_NAMES.len() as u32);
    let first = MOCK_NAME_SET[first as usize];
    let last = MOCK_LAST_NAMES[last as usize];
    return format!("{} {}", first, last);
}

const MOCK_NAME_SET: [&str; 32] = [
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel", "India", "Juliet",
    "Kilo", "Lima", "Mike", "November", "Oscar", "Papa", "Quebec", "Romeo", "Sierra", "Tango",
    "Uniform", "Victor", "Whiskey", "X-ray", "Yankee", "Zulu", "Red", "Blue", "Green", "Yellow",
    "Purple", "Orange",
];

const MOCK_LAST_NAMES: [&str; 16] = [
    "Warrior",
    "Ranger",
    "Mage",
    "Knight",
    "Assassin",
    "Paladin",
    "Druid",
    "Hunter",
    "Berserker",
    "Monk",
    "Ninja",
    "Samurai",
    "Viking",
    "Gladiator",
    "Sorcerer",
    "Alchemist",
];
