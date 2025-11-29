use game_state::{GameTrace, PlayerState};
use rusqlite::Transaction;
use serde::Serialize;
use std::future::Future;

use futures::channel::mpsc::{channel, Sender};

const DB_PATH: &str = "./data/game.db";

#[derive(Serialize)]
pub struct DBPlayer {
    name: String,
    kills: usize,
    deaths: usize,
}

impl DBPlayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kills: 0,
            deaths: 0,
        }
    }

    fn from_player_state(player: &PlayerState) -> Self {
        Self {
            name: player.name.clone(),
            kills: player.kills,
            deaths: player.deaths,
        }
    }
}

enum DbKind {
    InMemory,
    File(String),
}

pub struct GameDatabase {
    conn: rusqlite::Connection,
}

impl GameDatabase {
    fn in_memory() -> anyhow::Result<Self> {
        return Self::new(DbKind::InMemory);
    }

    pub fn file(path: impl Into<String>) -> anyhow::Result<Self> {
        return Self::new(DbKind::File(path.into()));
    }

    pub fn actor() -> (Sender<GameTrace>, impl Future<Output = ()>) {
        let (sender, mut receiver) = channel::<GameTrace>(999999999);
        let future = async move {
            let mut db = GameDatabase::new_prod().expect("Failed to create DB");
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let tx = db.conn.transaction().expect("Failed to create transaction");
                while let Ok(msg) = receiver.try_next() {
                    match msg {
                        Some(msg) => {
                            if let Err(e) = handle_message(msg, &tx) {
                                log::error!("Error handling DB message: {}", e);
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                if let Err(e) = tx.commit() {
                    log::error!("Error committing DB transaction: {}", e);
                }
            }
        };
        return (sender, future);
    }

    pub fn create_server(&mut self, server: &str, seed: u32) -> anyhow::Result<u64> {
        let experiment_id = get_experiment_id();
        let tx = self.conn.transaction()?;
        tx.execute(
            "insert into games (name, seed, experiment_id) values (?1, ?2, ?3)",
            rusqlite::params![server, seed, experiment_id],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id as u64)
    }

    pub fn new_prod() -> anyhow::Result<Self> {
        let kind = DbKind::File(DB_PATH.to_string());
        return Self::new(kind);
    }

    fn new(kind: DbKind) -> anyhow::Result<Self> {
        let conn = match kind {
            DbKind::InMemory => rusqlite::Connection::open_in_memory()?,
            DbKind::File(path) => {
                match rusqlite::Connection::open(path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Error opening DB file: {}", e);
                        return Err(anyhow::anyhow!(e));
                    }
                }
            }
        };

        Ok(Self { conn })
    }

    pub fn setup(&self) {
        let setup_sql = include_str!("./sql/setup.sql");
        match self.conn.execute_batch(setup_sql) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error setting up DB: {}", e);
            }
        }
        let experiment_id = get_experiment_id();
        let number_of_bots: i32 = std::env::var("BOT_COUNT")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0);
        let number_of_servers: i32 = std::env::var("INITIAL_SERVER_COUNT")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0);

        self.conn
            .execute(
                "insert or replace into experiments (id, number_of_bots, number_of_servers) values (?1, ?2, ?3)",
                rusqlite::params![experiment_id, number_of_bots, number_of_servers],
            )
            .expect("Failed to insert experiment data");
    }

    pub fn get_leaderboard(&self, n: usize) -> anyhow::Result<Vec<DBPlayer>> {
        let n = n.min(100);
        let mut stmt = self
            .conn
            .prepare("select * from players order by kills desc LIMIT ?1")?;
        let rows = stmt.query_map(rusqlite::params![n], |row| {
            Ok(DBPlayer {
                name: row.get(0)?,
                kills: row.get(1)?,
                deaths: row.get(2)?,
            })
        })?;

        let mut players = Vec::new();
        for player in rows {
            players.push(player?);
        }

        Ok(players)
    }

    fn get_player(&self, name: &str) -> anyhow::Result<DBPlayer> {
        todo!()
    }
}

fn handle_message(msg: GameTrace, tx: &Transaction<'_>) -> anyhow::Result<()> {
    match msg {
        GameTrace::ShipDestroyed {
            killer,
            owner,
            ship_id,
            frame,
            game_id,
        } => {
            tx.execute(
                "insert into ships_destroyed (ship_id, owner, killer, frame, game_id) values (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![ship_id, owner.as_u64(), killer.as_u64(), frame, game_id])?;
            return Ok(());
        }
        GameTrace::PlayerConnected {
            player_id,
            game_id,
            player_name,
            tick,
        } => {
            tx.execute(
                "insert into players (player_id, name, game_id, tick) values (?1, ?2, ?3, ?4)",
                rusqlite::params![player_id.as_u64(), player_name, game_id, tick],
            )?;
            return Ok(());
        }
        GameTrace::GameTick {
            game_id,
            tick,
            micros_elapsed,
        } => {
            tx.execute(
                "insert into game_ticks (game_id, tick, micros_elapsed) values (?1, ?2, ?3)",
                rusqlite::params![game_id, tick, micros_elapsed],
            )?;
            return Ok(());
        }
        GameTrace::ExperimentTick {
            experiment_id,
            tick,
            micros_elapsed,
        } => {
            tx.execute(
                "insert into experiment_ticks (experiment_id, tick, micros_elapsed) values (?1, ?2, ?3)",
                rusqlite::params![experiment_id, tick, micros_elapsed],
            )?;
            return Ok(());
        }
        GameTrace::PingTime {
            micros,
            player_id,
            tick,
        } => {
            tx.execute(
                "insert into pings (player_id, micros, tick) values (?1, ?2, ?3)",
                rusqlite::params![player_id.as_u64(), micros, tick],
            )?;
            return Ok(());
        }
        GameTrace::PlayerDisconnected {
            player_id,
            game_id,
            tick,
        } => {
            return Ok(());
        }
        GameTrace::NumberOfPlayers {
            game_id,
            count,
            tick,
        } => {
            tx.execute(
                "insert into player_counts (game_id, tick, count) values (?1, ?2, ?3)",
                rusqlite::params![game_id, tick, count as u64],
            )?;
            return Ok(());
        }
        GameTrace::ShipCreated {
            ship_id,
            owner,
            frame,
            game_id,
        } => {
            tx.execute(
                    "insert into ships_created (ship_id, owner, frame, game_id) values (?1, ?2, ?3, ?4)",
                    rusqlite::params![ship_id, owner.as_u64(), frame, game_id],
                )?;
            return Ok(());
        }
    }
}

pub fn get_experiment_id() -> u64 {
    let id = std::env::var("EXPERIMENT_ID").unwrap_or("0".to_string());
    return id.parse::<u64>().unwrap_or(0);
}
