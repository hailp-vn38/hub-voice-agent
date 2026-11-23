use rusqlite::Connection;
use std::path::Path;

/// Numbered schema migrations, applied in order. Each is idempotent.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/0001_init.sql"),
    include_str!("migrations/0002_countdowns.sql"),
    include_str!("migrations/0003_reminders.sql"),
    include_str!("migrations/0004_weather.sql"),
    include_str!("migrations/0005_photos.sql"),
    include_str!("migrations/0006_calendar.sql"),
    include_str!("migrations/0007_timer_duration.sql"),
    include_str!("migrations/0008_photo_source.sql"),
];

pub struct Store {
    pub(crate) connection: Connection,
    pub(crate) photos_dir: std::path::PathBuf,
    ephemeral_photos: bool,
}

impl Store {
    pub fn open(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir.as_ref())?;
        Self::initialize(
            Connection::open(dir.as_ref().join("hub.sqlite3"))?,
            dir.as_ref().join("photos"),
            false,
        )
    }

    pub fn in_memory() -> anyhow::Result<Self> {
        Self::initialize(
            Connection::open_in_memory()?,
            std::env::temp_dir().join(format!("ai-hub-photos-{}", uuid::Uuid::new_v4())),
            true,
        )
    }

    fn initialize(
        connection: Connection,
        photos_dir: std::path::PathBuf,
        ephemeral_photos: bool,
    ) -> anyhow::Result<Self> {
        connection.pragma_update(None, "journal_mode", "WAL")?;
        for migration in &MIGRATIONS[..MIGRATIONS.len() - 2] {
            connection.execute_batch(migration)?;
        }
        if !has_column(&connection, "timers", "duration_seconds")? {
            connection.execute_batch(MIGRATIONS[MIGRATIONS.len() - 2])?;
        }
        if !has_column(&connection, "photos", "source")? {
            connection.execute_batch(MIGRATIONS[MIGRATIONS.len() - 1])?;
        }
        Ok(Self {
            connection,
            photos_dir,
            ephemeral_photos,
        })
    }
}

fn has_column(connection: &Connection, table: &str, expected: &str) -> anyhow::Result<bool> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    Ok(columns
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|column| column == expected))
}

impl Drop for Store {
    fn drop(&mut self) {
        if self.ephemeral_photos {
            let _ = std::fs::remove_dir_all(&self.photos_dir);
        }
    }
}
