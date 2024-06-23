use crate::{ClientMessage, ServerState};
use std::collections::HashMap;
use tokio::sync::oneshot;
type ClientSender = tokio::sync::mpsc::Sender<String>;

pub enum GameMessage {
    NewConnection {
        sender: ClientSender,
        id_sender: oneshot::Sender<u64>,
    },
    ClientDisconnect(u64),
    ClientMessage(String),
}

pub struct CanvasGame {
    current_id: u64,
    players_senders: HashMap<u64, ClientSender>,
    game_state: ServerState,
}

impl CanvasGame {
    pub fn new() -> CanvasGame {
        CanvasGame {
            players_senders: HashMap::new(),
            game_state: ServerState::new(),
            current_id: 0,
        }
    }

    async fn send_message_to_player(&mut self, id: u64, message: ClientMessage) {
        if let Some(sender) = self.players_senders.get_mut(&id) {
            let msg = serde_json::to_string(&message).unwrap();
            match sender.send(msg).await {
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
        self.players_senders.insert(id, sender);
        id
    }

    async fn broadcast_message(&mut self, message: ClientMessage) {
        let player_ids: Vec<u64> = self.players_senders.keys().cloned().collect();
        for id in player_ids {
            self.send_message_to_player(id, message.clone()).await;
        }
    }

    pub async fn on_message(&mut self, msg: GameMessage) -> anyhow::Result<()> {
        match msg {
            GameMessage::ClientMessage(msg) => {
                let msg = self.game_state.on_string_message(msg)?;
                self.broadcast_message(msg).await;
            }
            GameMessage::NewConnection { sender, id_sender } => {
                let id = self.handle_create_player(sender).await;
                let msg = ClientMessage::CreatePlayer { id };
                let my_id = ClientMessage::MarkMyID { id };
                self.send_message_to_player(id, my_id).await;
                let state = self.game_state.state_message();
                self.send_message_to_player(id, state).await;
                self.game_state.on_message(msg.clone());
                self.broadcast_message(msg).await;
                id_sender.send(id).unwrap();
            }
            GameMessage::ClientDisconnect(id) => {
                self.players_senders.remove(&id);
                let msg = ClientMessage::RemovePlayer { id };
                self.game_state.on_message(msg.clone());
                self.broadcast_message(msg).await;
            }
        }
        Ok(())
    }
}
