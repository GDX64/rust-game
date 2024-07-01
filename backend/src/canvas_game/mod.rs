use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{stream::SplitSink, SinkExt};
use game_state::{GameMessage, GameServer};

type PlayerSender = SplitSink<WebSocket, Message>;

pub struct BackendServer {
    game_server: GameServer,
    player_channels: HashMap<u64, PlayerSender>,
}

impl BackendServer {
    pub fn new() -> BackendServer {
        BackendServer {
            game_server: GameServer::new(),
            player_channels: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, sender: PlayerSender) -> u64 {
        let id = self.game_server.next_player_id();
        self.game_server.new_connection(id);
        self.player_channels.insert(id, sender);
        return id;
    }

    pub async fn tick(&mut self) {
        self.game_server.tick();
        for (id, msg) in self.game_server.messages_to_send.drain(..) {
            if let Some(sender) = self.player_channels.get_mut(&id) {
                println!("sending message: {:?}", msg);
                if let Err(err) = sender.feed(Message::Text(msg.to_string())).await {
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

    pub fn on_string_message(&mut self, msg: String) {
        let game_message: GameMessage = msg.into();
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
