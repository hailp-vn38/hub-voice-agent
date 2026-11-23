use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Args {
    pub entity: String,
}
pub fn definitions() -> Vec<ToolDef> {
    [
        ("hub_home_get", "Get a device state"),
        ("hub_home_turn_on", "Turn on a device"),
        ("hub_home_turn_off", "Turn off a device"),
    ]
    .into_iter()
    .map(|(name, description)| ToolDef {
        name: name.into(),
        description: description.into(),
        input_schema: serde_json::to_value(schema_for!(Args)).unwrap(),
    })
    .collect()
}
