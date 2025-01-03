use super::game_server::GameMessage;
use super::local_client::Client;
use crate::server_state::{ServerState, StateMessage};
use crate::utils::event_hub::{EventHub, EventKey};
use crate::utils::vectors::V2D;
use crate::TICK_TIME;
use log::info;

#[derive(Debug, Clone, PartialEq)]
pub enum RunningEvent {
    MyID(u64),
    PositionChanged(V2D),
    Pong,
}

impl EventKey for RunningEvent {}

pub struct RunningMode {
    game_state: ServerState,
    client: Box<dyn Client>,
    frame_acc: f64,
    frame_buffer: Vec<Vec<StateMessage>>,
    player_id: u64,
    pub start_position: V2D,
    pub events: EventHub<RunningEvent>,
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        &self.game_state
    }

    pub fn new(client: Box<dyn Client>) -> RunningMode {
        RunningMode {
            game_state: ServerState::new(client.get_seed()),
            client,
            frame_acc: 0.0,
            frame_buffer: vec![],
            player_id: 0,
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
                    info!("My ID is: {}", id);
                    self.player_id = id;
                    self.start_position = V2D::new(x, y);
                    self.events.notify(RunningEvent::MyID(id));
                    self.events
                        .notify(RunningEvent::PositionChanged(self.start_position));
                }
                GameMessage::Reconnection => {
                    self.send_game_message(GameMessage::AskBroadcast { player: self.id() });
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
    }

    pub fn clear_flags(&mut self) {
        self.game_state.clear_flags();
    }

    pub fn id(&self) -> u64 {
        self.player_id
    }

    pub fn send_game_message(&mut self, msg: GameMessage) {
        self.client.send(msg);
    }
}

#[cfg(test)]
mod test {
    use crate::server::{game_server::GameMessage, local_client::LocalClient};

    #[test]
    fn running_mode() {
        let client = LocalClient::new("test_player".to_string(), 0);
        let mut local = super::RunningMode::new(Box::new(client));
        local.send_game_message(GameMessage::AddBot);
        local.send_game_message(GameMessage::AddBot);
        local.send_game_message(GameMessage::AddBot);
        local.send_game_message(GameMessage::AddBot);
        for _ in 0..1000 {
            local.tick(0.016)
        }
        assert_eq!(
            local.game_state.ship_collection.len(),
            local.client.server_state().unwrap().ship_collection.len()
        );
    }
}
