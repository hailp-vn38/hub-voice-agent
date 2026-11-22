use hub_core::UiState;
use slint::Weak;
use tokio::sync::watch;

use crate::App;

fn apply(app: &App, state: &UiState) {
    app.set_clock(state.clock.as_str().into());
}

/// Forwards each `UiState` snapshot to the Slint window on the event-loop thread.
pub fn spawn_bridge(app: Weak<App>, mut rx: watch::Receiver<UiState>) {
    tokio::spawn(async move {
        // Skip the initial snapshot so the window keeps its placeholder clock.
        rx.borrow_and_update();
        while rx.changed().await.is_ok() {
            let state = rx.borrow_and_update().clone();
            let app = app.clone();
            let queued = slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    apply(&app, &state);
                }
            });
            if queued.is_err() {
                return;
            }
        }
    });
}
