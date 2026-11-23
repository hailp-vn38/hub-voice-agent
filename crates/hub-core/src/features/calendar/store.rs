use crate::{CalendarEvent, Store};
impl Store {
    pub(crate) fn calendar_load(&self) -> anyhow::Result<(Vec<CalendarEvent>, Option<String>)> {
        let mut stmt = self
            .connection
            .prepare("SELECT events, token FROM calendar_cache WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => Ok((
                serde_json::from_str(&row.get::<_, String>(0)?)?,
                row.get(1)?,
            )),
            None => Ok((Vec::new(), None)),
        }
    }
    pub(crate) fn calendar_save(
        &self,
        events: &[CalendarEvent],
        token: &Option<String>,
    ) -> anyhow::Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO calendar_cache (id, events, token) VALUES (1, ?1, ?2)",
            rusqlite::params![serde_json::to_string(events)?, token],
        )?;
        Ok(())
    }
}
