use crate::{Screen, UiState};

pub(super) fn screen(state: &UiState) -> Screen {
    state.view.screen
}
