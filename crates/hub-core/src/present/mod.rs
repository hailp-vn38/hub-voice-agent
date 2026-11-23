mod ambient;
mod nav;
mod status;

use crate::UiState;
pub use ambient::AmbientView;
pub use status::StatusView;

/// Rust representation of the UI contract fields prepared by hub-core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubView {
    pub nav: crate::Screen,
    pub status: StatusView,
    pub ambient: AmbientView,
}

/// Prepares display-ready contract data from the actor snapshot.
pub fn present(state: &UiState) -> HubView {
    HubView {
        nav: nav::screen(state),
        status: status::from_state(state),
        ambient: ambient::from_state(state),
    }
}
