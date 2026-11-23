use crate::features::tool_args::definition;
use schemars::JsonSchema;
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageInput {
    pub url: String,
    pub caption: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Show {
    pub images: Vec<ImageInput>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub action: Action,
    pub id: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Open,
    Next,
    Previous,
    Pin,
    Unpin,
}

pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<Show>("hub_show_image", "Show one to six images from URLs"),
        definition::<List>("hub_image_list", "List cached images"),
        definition::<Control>("hub_image_control", "Open, navigate, pin or unpin an image"),
    ]
}
