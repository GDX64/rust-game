use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver},
};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{stream::SplitSink, SinkExt};
use game_state::{GameMessage, GameServer, MessageToSend};

type PlayerSender = SplitSink<WebSocket, Message>;

pub struct BackendServer {
    game_server: GameServer,
    player_channels: HashMap<u64, PlayerSender>,
    receiver: Receiver<MessageToSend>,
}

impl BackendServer {
    pub fn new() -> BackendServer {
        let (sender, receiver) = channel();
        BackendServer {
            game_server: GameServer::new(sender),
            player_channels: HashMap::new(),
            receiver,
        }
    }

    pub fn add_player(&mut self, sender: PlayerSender) -> u64 {
        let id = self.game_server.next_player_id();
        self.player_channels.insert(id, sender);
        self.game_server.new_connection(id);
        return id;
    }

    pub async fn tick(&mut self, dt: f64) {
        self.game_server.tick(dt);
        for (id, msg) in self.receiver.try_iter() {
            if let Some(sender) = self.player_channels.get_mut(&id) {
                if let Err(err) = sender.feed(Message::Binary(msg.to_bytes())).await {
                    eprintln!("error sending message: {:?}", err);
                }
            }
        }
        for sender in self.player_channels.values_mut() {
            if let Err(err) = sender.flush().await {
                eprintln!("error flushing message: {:?}", err);
            }
        }
    }

    pub fn on_string_message(&mut self, msg: Vec<u8>) {
        let game_message: GameMessage = GameMessage::from_bytes(&msg);
        match game_message {
            GameMessage::ClientMessage(client_message) => {
                self.game_server.on_message(client_message);
            }
            _ => {}
        }
    }

    pub fn disconnect_player(&mut self, id: u64) {
        self.game_server.disconnect_player(id);
        self.player_channels.remove(&id);
    }
}
