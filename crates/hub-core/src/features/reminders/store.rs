use super::{Reminder, ReminderHistory};
use crate::Store;

impl Store {
    pub(crate) fn reminders_load(&self) -> anyhow::Result<Vec<Reminder>> {
        let mut statement = self
            .connection
            .prepare("SELECT data FROM reminders ORDER BY id")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }
    pub(crate) fn reminder_history_load(&self) -> anyhow::Result<Vec<ReminderHistory>> {
        let mut statement = self
            .connection
            .prepare("SELECT data FROM reminder_history ORDER BY id")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }
    pub(crate) fn reminders_save(
        &mut self,
        items: &[Reminder],
        history: &[ReminderHistory],
    ) -> anyhow::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM reminders", [])?;
        for item in items {
            tx.execute(
                "INSERT INTO reminders (id, data) VALUES (?1, ?2)",
                rusqlite::params![item.id, serde_json::to_string(item)?],
            )?;
        }
        tx.execute("DELETE FROM reminder_history", [])?;
        for (id, item) in history.iter().enumerate() {
            tx.execute(
                "INSERT INTO reminder_history (id, data) VALUES (?1, ?2)",
                rusqlite::params![id as i64, serde_json::to_string(item)?],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
