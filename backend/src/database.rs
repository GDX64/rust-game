use game_state::{GameTrace, PlayerState};
use serde::Serialize;
use std::future::Future;

use futures::{
    channel::mpsc::{channel, Sender},
    StreamExt,
};

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

    pub fn actor(file: impl Into<String>) -> (Sender<GameTrace>, impl Future<Output = ()>) {
        let (sender, mut receiver) = channel::<GameTrace>(100);
        let future = async move {
            let mut db = GameDatabase::file(file).unwrap();
            while let Some(msg) = receiver.next().await {
                match msg {
                    GameTrace::ShipDestroyed {
                        killed_by,
                        player_id,
                        ship_id,
                        frame,
                        game_id,
                    } => {
                        db.handle_kill(killed_by.into(), player_id.into(), ship_id, frame, game_id)
                            .expect("Failed to update players");
                    }
                    _ => {}
                }
            }
        };
        return (sender, future);
    }

    fn new(kind: DbKind) -> anyhow::Result<Self> {
        let conn = match kind {
            DbKind::InMemory => rusqlite::Connection::open_in_memory()?,
            DbKind::File(path) => rusqlite::Connection::open(path)?,
        };

        conn.execute(
            "create table if not exists kills (
                 id integer primary key,
                 frame integer,
                 game_id integer,
                 ship_id integer,
                 player_id integer,
                 killed_by integer
             )",
            rusqlite::params![],
        )?;
        Ok(Self { conn })
    }

    fn insert_player(&self, player: &DBPlayer) -> anyhow::Result<()> {
        self.conn.execute(
            "insert or replace into players (name, kills, deaths) values (?1, ?2, ?3)",
            rusqlite::params![player.name, player.kills, player.deaths],
        )?;
        Ok(())
    }

    fn bulk_update_players(&mut self, players: &[DBPlayer]) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        for player in players {
            tx.execute(
                "insert or replace into players (name, kills, deaths) values (?1, ?2, ?3)",
                rusqlite::params![player.name, player.kills, player.deaths],
            )?;
        }
        tx.commit()?;
        Ok(())
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
        let mut stmt = self.conn.prepare("select * from players where name = ?1")?;
        let mut rows = stmt.query(rusqlite::params![name])?;
        let row = rows
            .next()?
            .ok_or_else(|| anyhow::anyhow!("Player not found"))?;

        let name: String = row.get(0)?;
        let kills: usize = row.get(1)?;
        let deaths: usize = row.get(2)?;

        let player = DBPlayer {
            name: name,
            kills: kills,
            deaths: deaths,
        };

        Ok(player)
    }
}

#[cfg(test)]
mod test {
    use crate::database::DBPlayer;

    use super::GameDatabase;

    #[test]
    fn test_db_start() {
        let db = GameDatabase::in_memory().unwrap();
        let player = DBPlayer::new("test");
        db.insert_player(&player).unwrap();
        let player = db.get_player("test").unwrap();
        assert_eq!(player.name, "test");
        assert_eq!(player.kills, 0);
        assert_eq!(player.deaths, 0);
    }
}
