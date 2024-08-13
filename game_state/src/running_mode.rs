use crate::ws_channel::WSChannel;
use crate::{game_server, GameMessage, ServerState, StateMessage, TICK_TIME};
use futures::channel::mpsc::{channel, Receiver};
use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    ws: WSChannel,
    game_state: ServerState,
    id: u64,
    frame_acc: f64,
    frame_buffer: Vec<Vec<StateMessage>>,
    send_buffer: Vec<GameMessage>,
}

pub trait Client {
    fn send(&mut self, msg: GameMessage);
    fn tick(&mut self, dt: f64);
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        OnlineClient {
            ws: WSChannel::new(url),
            game_state: ServerState::new(),
            id: 0,
            frame_acc: 0.0,
            frame_buffer: vec![],
            send_buffer: vec![],
        }
    }

    pub async fn init(&mut self) {
        while let Some(msg) = self.ws.next().await {
            let msg = GameMessage::from_arr_bytes(&msg);
            for msg in msg.into_iter() {
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

    fn flush_send_buffer(&mut self) {
        self.ws.send(GameMessage::serialize_arr(&self.send_buffer));
        self.send_buffer.clear();
    }
}

impl Client for OnlineClient {
    fn send(&mut self, msg: GameMessage) {
        self.send_buffer.push(msg);
    }

    fn tick(&mut self, dt: f64) {
        self.flush_send_buffer();
        loop {
            let msg = self.ws.receive();
            let msg = match msg {
                Some(msg) => msg,
                _ => break,
            };
            let msg = GameMessage::from_arr_bytes(&msg);
            msg.into_iter().for_each(|msg| {
                match msg {
                    GameMessage::FrameMessage(msg) => {
                        self.frame_buffer.insert(0, msg);
                    }
                    _ => {}
                }
            });
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
}

pub struct LocalClient {
    game: game_server::GameServer,
    receiver: Receiver<Vec<u8>>,
    state: ServerState,
    id: u64,
}

impl LocalClient {
    pub fn new() -> LocalClient {
        let (sender, receiver) = channel(100);
        let mut game = game_server::GameServer::new();
        let player_id = game.new_connection(sender);
        info!("Local server started");
        LocalClient {
            game,
            receiver,
            state: ServerState::new(),
            id: player_id,
        }
    }
}

impl Client for LocalClient {
    fn send(&mut self, msg: GameMessage) {
        self.game.on_message(GameMessage::serialize_arr(&vec![msg]));
    }

    fn tick(&mut self, dt: f64) {
        self.game.tick(dt);
        while let Ok(Some(msg)) = self.receiver.try_next() {
            let game_message = GameMessage::from_arr_bytes(&msg);
            for msg in game_message.into_iter() {
                match msg {
                    GameMessage::FrameMessage(msg) => {
                        msg.into_iter().for_each(|msg| self.state.on_message(msg));
                    }
                    _ => {}
                }
            }
        }
    }
}

pub enum RunningMode {
    Local(LocalClient),
    Online(OnlineClient),
}

impl RunningMode {
    pub fn server_state(&self) -> &ServerState {
        match self {
            RunningMode::Local(data) => &data.state,
            RunningMode::Online(data) => &data.game_state,
        }
    }

    pub fn start_local() -> RunningMode {
        RunningMode::Local(LocalClient::new())
    }

    pub fn tick(&mut self, dt: f64) {
        match self {
            RunningMode::Local(client) => {
                client.tick(dt);
            }
            RunningMode::Online(data) => {
                data.tick(dt);
            }
        };
    }

    pub fn id(&self) -> u64 {
        match self {
            RunningMode::Local(data) => data.id,
            RunningMode::Online(data) => data.id,
        }
    }

    pub fn send_game_message(&mut self, msg: GameMessage) {
        match self {
            RunningMode::Local(ref mut data) => {
                data.game.on_message(GameMessage::serialize_arr(&vec![msg]));
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
            super::RunningMode::Local(data) => {
                assert_eq!(
                    data.game.game_state.ship_collection,
                    data.state.ship_collection
                );
            }
            _ => panic!("Expected local mode"),
        }
    }
}
