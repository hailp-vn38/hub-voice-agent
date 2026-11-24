use crate::features::tool_args::definition;
use schemars::JsonSchema;
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
pub struct Get {
    /// City name. OMIT unless the user names a city; omitted means the hub's configured default location.
    pub location: Option<String>,
    /// Optional: a date `YYYY-MM-DD` to show that day's hours, or `aqi`. Omit for today.
    pub focus: Option<String>,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![definition::<Get>(
        "hub_weather_get",
        "Get weather. With no location argument it returns the hub's configured default location; only pass location when the user explicitly names another city (never pass the default city again). Answer briefly in one or two sentences: current condition, temperature and rain chance only. Give hourly, multi-day, UV, sunrise/sunset or air-quality details only when the user asks for them",
    )]
}
