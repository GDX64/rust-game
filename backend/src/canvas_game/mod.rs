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

    pub fn tick(&mut self) {
        self.game_server.tick();
        self.game_server
            .messages_to_send
            .drain(..)
            .for_each(|(id, msg)| {
                if let Some(sender) = self.player_channels.get_mut(&id) {
                    let client_message = GameMessage::ClientMessage(msg).to_string();
                    let _ = sender.send(Message::Text(client_message));
                }
            });
    }

    pub fn on_string_message(&mut self, msg: String) {
        self.game_server.on_message(msg.into());
    }

    pub fn disconnect_player(&mut self, id: u64) {
        self.game_server.disconnect_player(id);
        self.player_channels.remove(&id);
    }
}
