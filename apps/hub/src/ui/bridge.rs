use crate::App;
use hub_core::UiState;
use slint::Weak;
use tokio::sync::watch;

fn apply(app: &App, state: &UiState, url_changed: bool) {
    app.set_clock(state.now.strftime("%H:%M:%S").to_string().into());
    let voice = &state.voice_settings;
    if url_changed {
        app.set_ota_url(voice.ota_url.as_str().into());
    }
    app.set_device_id(voice.device_id.as_str().into());
    app.set_voice_status(voice.status_label.as_str().into());
    app.set_activation(voice.activation.as_deref().unwrap_or("").into());
    app.set_voice_error(voice.error.as_deref().unwrap_or("").into());
    app.set_enrolling(voice.enrolling);
}

pub fn spawn_bridge(app: Weak<App>, mut rx: watch::Receiver<UiState>) {
    tokio::spawn(async move {
        let mut shown_url = None;
        while rx.changed().await.is_ok() {
            let state = rx.borrow_and_update().clone();
            let url_changed = shown_url.as_ref() != Some(&state.voice_settings.ota_url);
            shown_url = Some(state.voice_settings.ota_url.clone());
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
        }
    });
}
