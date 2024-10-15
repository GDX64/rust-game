struct DBPlayer {
    name: String,
    kills: i32,
    deaths: i32,
}

impl DBPlayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kills: 0,
            deaths: 0,
        }
    }
}

enum DbKind {
    InMemory,
    File(String),
}

struct GameDatabase {
    conn: rusqlite::Connection,
}

impl GameDatabase {
    fn in_memory() -> anyhow::Result<Self> {
        return Self::new(DbKind::InMemory);
    }

    pub fn file(path: impl Into<String>) -> anyhow::Result<Self> {
        return Self::new(DbKind::File(path.into()));
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

    pub fn insert_player(&self, player: &DBPlayer) -> anyhow::Result<()> {
        self.conn.execute(
            "insert or replace into players (name, kills, deaths) values (?1, ?2, ?3)",
            rusqlite::params![player.name, player.kills, player.deaths],
        )?;
        Ok(())
    }

    pub fn bulk_update_players(&mut self, players: &[DBPlayer]) -> anyhow::Result<()> {
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

    pub fn get_player(&self, name: &str) -> anyhow::Result<DBPlayer> {
        let mut stmt = self.conn.prepare("select * from players where name = ?1")?;
        let mut rows = stmt.query(rusqlite::params![name])?;
        let row = rows
            .next()?
            .ok_or_else(|| anyhow::anyhow!("Player not found"))?;

        let name: String = row.get(0)?;
        let kills: i32 = row.get(1)?;
        let deaths: i32 = row.get(2)?;

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
