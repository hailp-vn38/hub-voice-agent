use crate::{ToolTrailStep, UiState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTrailView {
    pub steps: Vec<ToolTrailStep>,
}

pub(super) fn from_state(state: &UiState) -> ToolTrailView {
    ToolTrailView {
        steps: state.tool_trail.clone(),
    }
}
