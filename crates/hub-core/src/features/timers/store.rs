use super::rules::Timer;
use crate::Store;

impl Store {
    pub(crate) fn timers_load(&self) -> anyhow::Result<Vec<Timer>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, label, end_seconds, paused_seconds FROM timers ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Timer {
                id: row.get(0)?,
                label: row.get(1)?,
                end_seconds: row.get(2)?,
                paused_seconds: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub(crate) fn timers_save(&mut self, timers: &[Timer]) -> anyhow::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM timers", [])?;
        for timer in timers {
            tx.execute("INSERT INTO timers (id, label, end_seconds, paused_seconds) VALUES (?1, ?2, ?3, ?4)", rusqlite::params![timer.id, timer.label, timer.end_seconds, timer.paused_seconds])?;
        }
        tx.commit()?;
        Ok(())
    }
}
