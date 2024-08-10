use crate::ws_channel::WSChannel;
use crate::{game_server, GameMessage, ServerState, StateMessage};
use futures::channel::mpsc::{channel, Receiver};
use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    ws: WSChannel,
    game_state: ServerState,
    id: u64,
    frame_buffer: Vec<Vec<StateMessage>>,
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        OnlineClient {
            ws: WSChannel::new(url),
            game_state: ServerState::new(),
            id: 0,
            frame_buffer: vec![],
        }
    }

    pub async fn init(&mut self) {
        while let Some(msg) = self.ws.next().await {
            let msg = GameMessage::from_bytes(&msg);
            match msg {
                GameMessage::MyID(id) => {
                    info!("My ID is: {}", id);
                    self.id = id;
                    return ();
                }
                _ => {}
            }
        }
    }
}

impl OnlineClient {
    pub fn send(&mut self, msg: GameMessage) {
        self.ws.send(msg.to_bytes());
    }

    pub fn tick(&mut self, _dt: f64) {
        loop {
            let msg = self.ws.receive();
            let msg = match msg {
                Some(msg) => msg,
                _ => break,
            };
            let msg = GameMessage::from_bytes(&msg);
            match msg {
                GameMessage::FrameMessage(msg) => {
                    self.frame_buffer.push(msg);
                }
                _ => {}
            }
            if self.frame_buffer.len() > 2 {
                let frame = self.frame_buffer.pop().unwrap();
                frame
                    .into_iter()
                    .for_each(|msg| self.game_state.on_message(msg));
            }
        }
    }
}

pub enum RunningMode {
    Local {
        game: game_server::GameServer,
        receiver: Receiver<Vec<u8>>,
        state: ServerState,
        id: u64,
    },
    Online(OnlineClient),
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        match self {
            RunningMode::Local { ref state, .. } => state,
            RunningMode::Online(data) => &data.game_state,
        }
    }

    pub fn start_local() -> RunningMode {
        let (sender, receiver) = channel(100);
        let mut game = game_server::GameServer::new();
        let player_id = game.new_connection(sender);
        info!("Local server started");
        return RunningMode::Local {
            game,
            receiver,
            id: player_id,
            state: ServerState::new(),
        };
    }

    pub fn tick(&mut self, dt: f64) {
        match self {
            RunningMode::Local {
                receiver,
                state,
                game,
                ..
            } => {
                game.tick(dt);
                while let Ok(Some(msg)) = receiver.try_next() {
                    info!("Received message {:?}", msg);
                    let game_message = GameMessage::from_bytes(&msg);
                    match game_message {
                        GameMessage::FrameMessage(msg) => {
                            msg.into_iter().for_each(|msg| state.on_message(msg));
                        }
                        _ => {}
                    }
                }
            }
            RunningMode::Online(data) => {
                data.tick(dt);
            }
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local { id, .. } => *id,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn send_game_message(&mut self, msg: GameMessage) {
        match self {
            RunningMode::Local { ref mut game, .. } => {
                game.on_message(msg.to_bytes());
            }
            RunningMode::Online(data) => {
                data.send(msg);
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn running_mode() {
        let mut local = super::RunningMode::start_local();
        local.send_game_message(crate::GameMessage::AddBot);
        local.send_game_message(crate::GameMessage::AddBot);
        for _ in 0..1000 {
            local.tick(0.016)
        }
        match local {
            super::RunningMode::Local { game, state, .. } => {
                assert_eq!(game.game_state.ship_collection, state.ship_collection);
            }
            _ => panic!("Expected local mode"),
        }
    }
}
