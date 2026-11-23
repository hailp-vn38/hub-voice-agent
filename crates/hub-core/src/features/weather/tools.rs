use crate::features::tool_args::definition;
use schemars::JsonSchema;
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Get {
    pub location: Option<String>,
    pub focus: Option<String>,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![definition::<Get>(
        "hub_weather_get",
        "Get weather for a city or the home location",
    )]
}
