use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{stream::SplitSink, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

struct Player {
    name: String,
    position: (f64, f64),
    sender: ClientSender,
    id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlayerState {
    name: String,
    position: (f64, f64),
    id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadCastState {
    players: Vec<PlayerState>,
}

type ClientSender = SplitSink<WebSocket, Message>;

pub enum GameMessage {
    NewConnection {
        sender: ClientSender,
        id_sender: oneshot::Sender<u64>,
    },
    ClientDisconnect(u64),
    ClientMessage(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMessage {
    SetPlayerName { name: String, id: u64 },
    PlayerCreatedResponse { id: u64 },
    MovePlayer { position: (f64, f64), id: u64 },
    BroadCastState { state: BroadCastState },
}

pub struct CanvasGame {
    current_id: u64,
    players: HashMap<u64, Player>,
}

impl CanvasGame {
    pub fn new() -> CanvasGame {
        CanvasGame {
            players: HashMap::new(),
            current_id: 0,
        }
    }

    pub async fn run(mut channel: tokio::sync::mpsc::Receiver<GameMessage>) {
        let mut game = CanvasGame::new();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    game.broadcast_state().await;
                }
                Some(msg) = channel.recv() => {
                    match game.on_message(msg).await  {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    async fn send_message_to_player(&mut self, id: u64, message: ClientMessage) {
        if let Some(player) = self.players.get_mut(&id) {
            let msg = serde_json::to_string(&message).unwrap();
            match player.sender.send(Message::Text(msg)).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending message: {:?}", e);
                }
            }
        }
    }

    async fn handle_create_player(&mut self, sender: ClientSender) -> u64 {
        let id = self.current_id;
        self.current_id += 1;
        self.players.insert(
            id,
            Player {
                name: format!("Player {}", id),
                position: (0.0, 0.0),
                id,
                sender,
            },
        );
        self.send_message_to_player(id, ClientMessage::PlayerCreatedResponse { id })
            .await;
        id
    }

    fn handle_move_player(&mut self, id: u64, position: (f64, f64)) {
        if let Some(player) = self.players.get_mut(&id) {
            player.position = position;
        }
    }

    fn handle_set_player_name(&mut self, name: String, id: u64) {
        if let Some(player) = self.players.get_mut(&id) {
            player.name = name;
        }
    }

    async fn broadcast_state(&mut self) {
        let states = self
            .players
            .values()
            .map(|player| {
                PlayerState {
                    name: player.name.clone(),
                    position: player.position,
                    id: player.id,
                }
            })
            .collect();
        let state = BroadCastState { players: states };
        let msg = ClientMessage::BroadCastState { state };
        let msg = serde_json::to_string(&msg).unwrap();
        for player in self.players.values_mut() {
            match player.sender.send(Message::Text(msg.clone())).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending message: {:?}", e);
                }
            }
        }
    }

    async fn on_message(&mut self, msg: GameMessage) -> anyhow::Result<()> {
        match msg {
            GameMessage::ClientMessage(msg) => {
                println!("Received message: {:?}", msg);
                let client_msg: ClientMessage = serde_json::from_str(&msg)?;
                match client_msg {
                    ClientMessage::SetPlayerName { name, id } => {
                        self.handle_set_player_name(name, id);
                    }
                    ClientMessage::MovePlayer { position, id } => {
                        println!("Move player: {:?}", position);
                        self.handle_move_player(id, position);
                    }
                    _ => {}
                }
            }
            GameMessage::NewConnection { sender, id_sender } => {
                let id = self.handle_create_player(sender).await;
                id_sender.send(id).unwrap();
            }
            GameMessage::ClientDisconnect(id) => {
                self.players.remove(&id);
            }
        }
        Ok(())
    }
}
