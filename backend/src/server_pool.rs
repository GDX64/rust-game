use anyhow::Result;
use game_state::GameServer;
use std::{collections::HashMap, error::Error};

const MAX_SERVERS: usize = 3;

pub struct ServerPool {
    servers: HashMap<String, GameServer>,
}

#[derive(serde::Serialize)]
pub struct ServerInfo {
    name: String,
    players: usize,
}

impl ServerPool {
    pub fn new() -> ServerPool {
        ServerPool {
            servers: HashMap::new(),
        }
    }

    pub fn get_server(&mut self, server_id: &str) -> Option<&mut GameServer> {
        self.servers.get_mut(server_id)
    }

    pub fn tick(&mut self, time: f64) {
        for (_, server) in self.servers.iter_mut() {
            server.tick(time);
        }
    }

    pub fn get_server_info(&self) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .map(|(name, server)| {
                ServerInfo {
                    name: name.clone(),
                    players: server.get_player_count(),
                }
            })
            .collect()
    }

    pub fn remove_server(&mut self, server_id: &str) -> Result<()> {
        if self.servers.remove(server_id).is_none() {
            return Err(anyhow::anyhow!("Server not found"));
        }
        return Ok(());
    }

    pub fn create_server(&mut self, server_id: &str) -> Result<()> {
        if self.servers.len() >= MAX_SERVERS {
            return Err(anyhow::anyhow!("Max servers reached"));
        }
        let mut server = GameServer::new();
        server.name = server_id.to_string();
        self.servers.insert(server_id.to_string(), server);
        return Ok(());
    }
}
