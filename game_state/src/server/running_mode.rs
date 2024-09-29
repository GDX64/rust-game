use super::game_server;
use crate::utils::vectors::V2D;
use crate::wasm_game::{GameMessage, ServerState, StateMessage};
use crate::TICK_TIME;
use futures::channel::mpsc::{channel, Receiver};
use log::info;
use wasm_bindgen::prelude::*;

pub trait Client {
    fn send(&mut self, msg: GameMessage);
    fn tick(&mut self, dt: f64);
    fn next_message(&mut self) -> Option<GameMessage>;
    fn server_state(&self) -> Option<&ServerState>;
}

#[wasm_bindgen]
pub struct LocalClient {
    game: game_server::GameServer,
    receiver: Receiver<Vec<u8>>,
    receive_buffer: Vec<GameMessage>,
}

#[wasm_bindgen]
impl LocalClient {
    pub fn new() -> LocalClient {
        let (sender, receiver) = channel(100);
        let mut game = game_server::GameServer::new();
        game.new_connection(sender, None);
        info!("Local server started");
        LocalClient {
            game,
            receiver,
            receive_buffer: vec![],
        }
    }
}

impl Client for LocalClient {
    fn send(&mut self, msg: GameMessage) {
        self.game.on_message(GameMessage::serialize_arr(&vec![msg]));
    }

    fn tick(&mut self, dt: f64) {
        self.game.tick(dt);
    }

    fn server_state(&self) -> Option<&ServerState> {
        Some(&self.game.game_state)
    }

    fn next_message(&mut self) -> Option<GameMessage> {
        if !self.receive_buffer.is_empty() {
            return Some(self.receive_buffer.remove(0));
        }
        match self.receiver.try_next() {
            Ok(Some(msg)) => {
                let game_message = GameMessage::from_arr_bytes(&msg);
                for msg in game_message.into_iter() {
                    self.receive_buffer.push(msg);
                }
                return self.next_message();
            }
            _ => None,
        }
    }
}

pub struct RunningMode {
    game_state: ServerState,
    client: Box<dyn Client>,
    frame_acc: f64,
    frame_buffer: Vec<Vec<StateMessage>>,
    player_id: u64,
    pub start_position: V2D,
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        &self.game_state
    }

    pub fn new(client: Box<dyn Client>) -> RunningMode {
        RunningMode {
            game_state: ServerState::new(),
            client,
            frame_acc: 0.0,
            frame_buffer: vec![],
            player_id: 0,
            start_position: V2D::new(0.0, 0.0),
        }
    }

    pub fn tick(&mut self, dt: f64) {
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
                    self.send_game_message(GameMessage::AskBroadcast { player: id });
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
    use crate::wasm_game::GameMessage;

    #[test]
    fn running_mode() {
        let client = super::LocalClient::new();
        let mut local = super::RunningMode::new(Box::new(client));
        local.send_game_message(GameMessage::AddBot);
        local.send_game_message(GameMessage::AddBot);
        for _ in 0..1000 {
            local.tick(0.016)
        }
        assert_eq!(
            local.game_state.ship_collection,
            local.client.server_state().unwrap().ship_collection
        );
    }
}
