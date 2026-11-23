use serde_json::Value;

impl crate::Store {
    pub(crate) fn weather_get(&self, key: &str) -> anyhow::Result<Option<(i64, Value)>> {
        use rusqlite::OptionalExtension;
        let row: Option<(i64, String)> = self
            .connection
            .query_row(
                "SELECT updated, data FROM weather_cache WHERE key = ?1",
                [key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        row.map(|(updated, data)| Ok((updated, serde_json::from_str(&data)?)))
            .transpose()
    }
    pub(crate) fn weather_save(&self, key: &str, updated: i64, data: &Value) -> anyhow::Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO weather_cache (key, updated, data) VALUES (?1, ?2, ?3)",
            rusqlite::params![key, updated, data.to_string()],
        )?;
        Ok(())
    }
}
