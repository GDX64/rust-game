use anyhow::Result;
use game::get_experiment_id;
use game_state::{GameServer, GameTrace};
use std::{collections::HashMap, time::Duration};

use crate::database::GameDatabase;

const MAX_SERVERS: usize = 100;

pub struct ServerPool {
    servers: HashMap<String, GameServer>,
    db: GameDatabase,
    tick_number: u64,
    experiment_id: u64,
}

#[derive(serde::Serialize)]
pub struct ServerInfo {
    name: String,
    players: usize,
    seed: u32,
}

impl ServerPool {
    pub fn new() -> ServerPool {
        let db = GameDatabase::new_prod().expect("Failed to create db");
        db.setup();
        ServerPool {
            servers: HashMap::new(),
            tick_number: 0,
            db,
            experiment_id: get_experiment_id(),
        }
    }

    pub fn get_server(&mut self, server_id: &str) -> Option<&mut GameServer> {
        self.servers.get_mut(server_id)
    }

    pub fn tick(&mut self, dt: f64) {
        let elapsed = measure_time(|| {
            for (_, server) in self.servers.iter_mut() {
                let elapsed = measure_time(|| {
                    server.tick(dt, self.tick_number);
                });
                GameTrace::GameTick {
                    game_id: server.game_id,
                    tick: self.tick_number,
                    micros_elapsed: elapsed.as_micros() as u64,
                }
                .send();
            }
            self.tick_number += 1;
        });
        GameTrace::ExperimentTick {
            tick: self.tick_number,
            micros_elapsed: elapsed.as_micros() as u64,
            experiment_id: self.experiment_id,
        }
        .send();
    }

    pub fn get_server_info(&self) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .map(|(name, server)| {
                ServerInfo {
                    name: name.clone(),
                    players: server.get_player_count(),
                    seed: server.seed,
                }
            })
            .collect()
    }

    pub fn get_player_id_for_server(&mut self, server_id: &str) -> Option<u64> {
        let server = self.servers.get_mut(server_id)?;
        return Some(server.next_player_id().into());
    }

    pub fn remove_server(&mut self, server_id: &str) -> Result<()> {
        if self.servers.remove(server_id).is_none() {
            return Err(anyhow::anyhow!("Server not found"));
        }
        return Ok(());
    }

    pub fn create_server(&mut self, server_name: &str, seed: u32) -> Result<()> {
        if self.servers.len() >= MAX_SERVERS {
            return Err(anyhow::anyhow!("Max servers reached"));
        }
        log::info!("Creating server {server_name} with seed {seed}");
        let id = self.db.create_server(&server_name, seed)?;
        let mut server = GameServer::new(seed, id);
        server.name = server_name.to_string();
        self.servers.insert(server_name.to_string(), server);
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
