use game_state::PlayerState;
use std::future::Future;

use futures::{
    channel::mpsc::{channel, Sender},
    StreamExt,
};

struct DBPlayer {
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

pub enum DBMessage {
    BulkInsert(Vec<PlayerState>),
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

    pub fn actor(file: impl Into<String>) -> (Sender<DBMessage>, impl Future<Output = ()>) {
        let (sender, mut receiver) = channel::<DBMessage>(100);
        let future = async move {
            let mut db = GameDatabase::file(file).unwrap();
            while let Some(msg) = receiver.next().await {
                match msg {
                    DBMessage::BulkInsert(players) => {
                        log::info!("Received stats update request: {:?}", players.len());
                        let players = players
                            .iter()
                            .map(DBPlayer::from_player_state)
                            .collect::<Vec<_>>();
                        db.bulk_update_players(&players)
                            .expect("Failed to update players");
                    }
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
            "create table if not exists players (
                 name text primary key,
                 kills integer,
                 deaths integer
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
