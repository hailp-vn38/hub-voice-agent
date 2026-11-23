use hub::voice_settings::SettingsState;
use slint::Weak;
use tokio::sync::watch;

use crate::App;

fn apply(app: &App, state: &SettingsState, url_changed: bool) {
    // `ota-url` is bound to the text field: only overwrite it when the saved
    // value changed, so typing is not clobbered by status updates.
    if url_changed {
        app.set_ota_url(state.ota_url.as_str().into());
    }
    app.set_device_id(state.device_id.as_str().into());
    app.set_voice_status(state.status_label().into());
    app.set_activation(state.activation.as_deref().unwrap_or("").into());
    app.set_voice_error(state.error.as_deref().unwrap_or("").into());
    app.set_enrolling(state.is_enrolling());
}

pub fn spawn_voice_bridge(app: Weak<App>, mut rx: watch::Receiver<SettingsState>) {
    tokio::spawn(async move {
        let mut shown_url = None;
        loop {
            let state = rx.borrow_and_update().clone();
            let url_changed = shown_url.as_ref() != Some(&state.ota_url);
            shown_url = Some(state.ota_url.clone());
            let app = app.clone();
            if slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    apply(&app, &state, url_changed);
                }
            })
            .is_err()
            {
                return;
            }
            if rx.changed().await.is_err() {
                return;
            }
        }
    });
}
