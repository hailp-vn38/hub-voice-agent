use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Get {
    pub location: Option<String>,
    pub focus: Option<String>,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "hub_weather_get".into(),
        description: "Get weather for a city or the home location".into(),
        input_schema: serde_json::to_value(schema_for!(Get)).unwrap(),
    }]
}
