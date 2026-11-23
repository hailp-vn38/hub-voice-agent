use rusqlite::Connection;
use std::path::Path;

pub struct Store {
    pub(crate) connection: Connection,
}

impl Store {
    pub fn open(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir.as_ref())?;
        Self::initialize(Connection::open(dir.as_ref().join("hub.sqlite3"))?)
    }

    pub fn in_memory() -> anyhow::Result<Self> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> anyhow::Result<Self> {
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.execute_batch(include_str!("migrations/0001_init.sql"))?;
        connection.execute_batch(include_str!("migrations/0002_countdowns.sql"))?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS calendar_cache (id INTEGER PRIMARY KEY, events TEXT NOT NULL, token TEXT)")?;
        connection.execute_batch(include_str!("migrations/0003_weather.sql"))?;
        Ok(Self { connection })
    }
}
