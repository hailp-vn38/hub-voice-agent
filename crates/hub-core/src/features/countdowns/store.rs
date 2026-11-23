use super::rules::{Countdown, Repeat};
use crate::Store;

impl Store {
    pub(crate) fn countdowns_load(&self) -> anyhow::Result<Vec<Countdown>> {
        let mut stmt = self.connection.prepare(
            "SELECT c.id, c.label, c.date, c.repeat, c.lunar_leap, COALESCE(m.created_at, '') \
                 FROM countdowns c LEFT JOIN countdown_metadata m ON m.countdown_id = c.id \
                 ORDER BY c.id",
        )?;
        let rows = stmt.query_map([], |row| {
            let repeat: String = row.get(3)?;
            let repeat = serde_json::from_str::<Repeat>(&format!("\"{repeat}\"")).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Countdown {
                id: row.get(0)?,
                label: row.get(1)?,
                date: row.get(2)?,
                repeat,
                lunar_leap: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub(crate) fn countdowns_save(&mut self, items: &[Countdown]) -> anyhow::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM countdowns", [])?;
        tx.execute("DELETE FROM countdown_metadata", [])?;
        for item in items {
            let repeat = serde_json::to_value(&item.repeat)?;
            tx.execute("INSERT INTO countdowns (id, label, date, repeat, lunar_leap) VALUES (?1, ?2, ?3, ?4, ?5)", rusqlite::params![item.id, item.label, item.date, repeat.as_str(), item.lunar_leap])?;
            if !item.created_at.is_empty() {
                tx.execute(
                    "INSERT INTO countdown_metadata (countdown_id, created_at) VALUES (?1, ?2)",
                    rusqlite::params![item.id, item.created_at],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}
