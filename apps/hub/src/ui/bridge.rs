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
        loop {
            let state = rx.borrow_and_update().clone();
            let app = app.clone();
            if slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    apply(&app, &state);
                }
            })
            .is_err()
                || rx.changed().await.is_err()
            {
                return;
            }
        }
    });
}
