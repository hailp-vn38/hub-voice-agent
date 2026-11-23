use super::rules::{Countdown, Repeat};
use crate::Store;

impl Store {
    pub(crate) fn countdowns_load(&self) -> anyhow::Result<Vec<Countdown>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, label, date, repeat, lunar_leap FROM countdowns ORDER BY id")?;
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
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub(crate) fn countdowns_save(&mut self, items: &[Countdown]) -> anyhow::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM countdowns", [])?;
        for item in items {
            let repeat = serde_json::to_value(&item.repeat)?;
            tx.execute("INSERT INTO countdowns (id, label, date, repeat, lunar_leap) VALUES (?1, ?2, ?3, ?4, ?5)", rusqlite::params![item.id, item.label, item.date, repeat.as_str(), item.lunar_leap])?;
        }
        tx.commit()?;
        Ok(())
    }
}
