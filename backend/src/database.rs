use game_state::{GameTrace, PlayerState};
use serde::Serialize;
use std::{fs, future::Future};

use futures::{
    channel::mpsc::{channel, Sender},
    StreamExt,
};

const DB_PATH: &str = "./data/game2.db";

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
        let (sender, mut receiver) = channel::<GameTrace>(100);
        let future = async move {
            let mut db = GameDatabase::new_prod().unwrap();
            while let Some(msg) = receiver.next().await {
                if let Err(e) = db.handle_message(msg) {
                    eprintln!("Error handling DB message: {}", e);
                }
            }
        };
        return (sender, future);
    }

    pub fn create_server(&mut self, server: &str, seed: u32) -> anyhow::Result<u64> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "insert into servers (name, seed) values (?1, ?2)",
            rusqlite::params![server, seed],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id as u64)
    }

    fn handle_message(&mut self, msg: GameTrace) -> anyhow::Result<()> {
        match msg {
            GameTrace::ShipDestroyed {
                killed_by,
                player_id,
                ship_id,
                frame,
                game_id,
            } => {
                return self.handle_kill(
                    killed_by.into(),
                    player_id.into(),
                    ship_id,
                    frame,
                    game_id,
                );
            }
            GameTrace::PlayerConnected {
                player_id,
                game_id,
                player_name,
            } => {
                let tx = self.conn.transaction()?;
                tx.execute(
                    "insert or ignore into players (player_id, name, game_id) values (?1, ?2, ?3)",
                    rusqlite::params![player_id.as_u64(), player_name, game_id],
                )?;
                tx.commit()?;
                return Ok(());
            }
            GameTrace::ServerTick {
                game_id,
                tick,
                micros_elapsed,
            } => {
                let tx = self.conn.transaction()?;
                tx.execute(
                    "insert into server_ticks (game_id, tick, micros_elapsed) values (?1, ?2, ?3)",
                    rusqlite::params![game_id, tick, micros_elapsed],
                )?;
                tx.commit()?;
                return Ok(());
            }
            GameTrace::PingTime { micros, player_id } => {
                let tx = self.conn.transaction()?;
                tx.execute(
                    "insert into pings (player_id, micros) values (?1, ?2)",
                    rusqlite::params![player_id.as_u64(), micros],
                )?;
                tx.commit()?;
                return Ok(());
            }
            _ => {
                return Ok(());
            }
        }
    }

    pub fn new_prod() -> anyhow::Result<Self> {
        let kind = DbKind::File(DB_PATH.to_string());
        return Self::new(kind);
    }

    fn new(kind: DbKind) -> anyhow::Result<Self> {
        let conn = match kind {
            DbKind::InMemory => rusqlite::Connection::open_in_memory()?,
            DbKind::File(path) => rusqlite::Connection::open(path)?,
        };

        let setup_sql = include_str!("./sql/setup.sql");
        conn.execute_batch(setup_sql)?;

        Ok(Self { conn })
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

    fn handle_kill(
        &mut self,
        killed_by: u64,
        player_id: u64,
        ship_id: u64,
        frame: u64,
        game_id: u64,
    ) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "insert into kills (ship_id, player_id, killed_by, frame, game_id) values (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![ship_id, player_id, killed_by, frame, game_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn get_player(&self, name: &str) -> anyhow::Result<DBPlayer> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::database::DBPlayer;

    use super::GameDatabase;

    #[test]
    fn test_db_start() {
        // let db = GameDatabase::in_memory().unwrap();
        // let player = DBPlayer::new("test");
        // db.insert_player(&player).unwrap();
        // let player = db.get_player("test").unwrap();
        // assert_eq!(player.name, "test");
        // assert_eq!(player.kills, 0);
        // assert_eq!(player.deaths, 0);
    }
}
