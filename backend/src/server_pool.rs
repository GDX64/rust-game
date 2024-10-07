use anyhow::Result;
use game_state::GameServer;
use std::{collections::HashMap, time::Duration};

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

    pub fn tick(&mut self, dt: f64) {
        let elapsed = measure_time(|| {
            for (_, server) in self.servers.iter_mut() {
                let elapsed = measure_time(|| {
                    server.tick(dt);
                });
                if elapsed.as_millis() > 16 {
                    let server_name = server.name.as_str();
                    log::warn!(
                        "Tick of server {server_name} took longer than a frame time: {}ms",
                        elapsed.as_millis()
                    );
                }
            }
        });
        if elapsed.as_millis() > 16 {
            log::warn!(
                "Tick took longer than a frame time: {}ms",
                elapsed.as_millis()
            );
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

fn measure_time(func: impl FnOnce()) -> Duration {
    let time_start = std::time::Instant::now();
    func();
    let time_end = std::time::Instant::now();
    let elapsed = time_end - time_start;
    return elapsed;
}
