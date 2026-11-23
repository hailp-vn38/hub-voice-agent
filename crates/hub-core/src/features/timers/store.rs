use super::rules::Timer;
use crate::Store;

impl Store {
    pub(crate) fn timers_load(&mut self, now: i64) -> anyhow::Result<Vec<Timer>> {
        self.connection.execute(
            "UPDATE timers SET duration_seconds = MAX(
                CASE WHEN paused_seconds IS NULL THEN end_seconds - ?1 ELSE paused_seconds END,
                1
            ) WHERE duration_seconds IS NULL",
            [now],
        )?;
        let mut stmt = self.connection.prepare(
            "SELECT id, label, duration_seconds, end_seconds, paused_seconds FROM timers ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Timer {
                id: row.get(0)?,
                label: row.get(1)?,
                duration_seconds: row.get(2)?,
                end_seconds: row.get(3)?,
                paused_seconds: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub(crate) fn timers_save(&mut self, timers: &[Timer]) -> anyhow::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM timers", [])?;
        for timer in timers {
            tx.execute(
                "INSERT INTO timers (id, label, end_seconds, paused_seconds, duration_seconds) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![timer.id, timer.label, timer.end_seconds, timer.paused_seconds, timer.duration_seconds],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
