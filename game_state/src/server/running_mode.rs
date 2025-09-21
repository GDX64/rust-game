use super::game_server::GameMessage;
use crate::player::Player;
use crate::player_state::PlayerID;
use crate::server::Client;
use crate::server_state::{ServerState, StateMessage};
use crate::utils::event_hub::{EventHub, EventKey};
use crate::utils::vectors::V2D;
use crate::{BotPlayer, TICK_TIME};
use log::info;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RunningEvent {
    PlayerCreated { id: PlayerID, x: f64, y: f64 },
    PositionChanged(V2D),
    Pong,
}

impl EventKey for RunningEvent {}

pub struct RunningMode {
    game_state: ServerState,
    client: Box<dyn Client>,
    frame_acc: f64,
    frame_buffer: Vec<Vec<StateMessage>>,
    players: BTreeMap<PlayerID, Player>,
    bots: BTreeMap<PlayerID, BotPlayer>,
    pub start_position: V2D,
    pub events: EventHub<RunningEvent>,
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        &self.game_state
    }

    pub fn new(client: Box<dyn Client>) -> RunningMode {
        RunningMode {
            game_state: ServerState::new(0),
            client,
            frame_acc: 0.0,
            frame_buffer: vec![],
            players: BTreeMap::new(),
            bots: BTreeMap::new(),
            start_position: V2D::new(0.0, 0.0),
            events: EventHub::new(),
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.client.tick(dt);
        loop {
            let msg = self.client.next_message();
            let msg = match msg {
                Some(msg) => msg,
                _ => break,
            };
            match msg {
                GameMessage::FrameMessage(msg) => {
                    self.frame_buffer.insert(0, msg);
                }
                GameMessage::PlayerCreated { id, x, y } => {
                    info!("Player Created with id: {:?}", id);
                    self.events.notify(RunningEvent::PlayerCreated { id, x, y });
                    self.events
                        .notify(RunningEvent::PositionChanged(self.start_position));
                }
                GameMessage::Reconnection { id, seed } => {
                    self.game_state = ServerState::new(seed);
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

        self.frame_acc += dt;
        let completed_frames = (self.frame_acc / TICK_TIME).round();
        self.frame_acc -= (completed_frames) * TICK_TIME;

        for _ in 0..completed_frames as usize {
            loop {
                if let Some(frame) = self.frame_buffer.pop() {
                    frame
                        .into_iter()
                        .for_each(|msg| self.game_state.on_message(msg));
                }
                if self.frame_buffer.len() < 10 {
                    break;
                }
            }
        }

        let state = &self.game_state;
        let mut messages_to_send = vec![];
        let mut bots_to_remove = vec![];
        for bot in self.bots.values_mut() {
            bot.tick(dt, state);
            bot.player.collect_messages().into_iter().for_each(|msg| {
                messages_to_send.push(msg);
            });
            if bot.is_dead() {
                println!("bot is dead");
                bots_to_remove.push(bot.player.id);
            }
        }
        for id in bots_to_remove {
            self.bots.remove(&id);
        }
        for msg in messages_to_send {
            self.send_game_message(GameMessage::InputMessage(msg));
        }
    }

    pub async fn create_bot(&mut self) {
        if let Ok((id, x, y)) = self._create_player().await {
            let mut bot = BotPlayer::new(id);
            bot.player.position = V2D::new(x, y);
            self.bots.insert(id, bot);
        }
    }

    pub async fn create_player(&mut self) {
        if let Ok((player_id, x, y)) = self._create_player().await {
            let mut player = Player::new(player_id);
            player.position = V2D::new(x, y);
            self.players.insert(player_id, player);
        }
    }

    async fn _create_player(&mut self) -> anyhow::Result<(PlayerID, f64, f64)> {
        self.send_game_message(GameMessage::CreatePlayer {
            name: None,
            flag: None,
        });
        let player_id = self
            .events
            .when(|e| {
                let RunningEvent::PlayerCreated { id, x, y } = e else {
                    return None;
                };
                return Some((id, x, y));
            })
            .await;
        return player_id;
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

#[cfg(test)]
mod test {
    // use crate::server::{game_server::GameMessage, local_client::LocalClient};

    #[test]
    fn running_mode() {
        // let client = LocalClient::new("test_player".to_string(), 0, Some("us".to_string()));
        // let mut local = super::RunningMode::new(Box::new(client));
        // local.send_game_message(GameMessage::AddBot);
        // local.send_game_message(GameMessage::AddBot);
        // local.send_game_message(GameMessage::AddBot);
        // local.send_game_message(GameMessage::AddBot);
        // for _ in 0..1000 {
        //     local.tick(0.016)
        // }
        // assert_eq!(
        //     local.game_state.ship_collection.len(),
        //     local.client.server_state().unwrap().ship_collection.len()
        // );
    }
}
