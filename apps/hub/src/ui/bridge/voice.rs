use crate::{App, ConnectionSettings, SettingsData, StatusBar};
use hub_core::UiState;
use slint::{ComponentHandle, Weak};
use tokio::sync::watch;

fn apply(app: &App, state: &UiState, url_changed: bool) {
    app.global::<StatusBar>().set_time(state.now.strftime("%H:%M").to_string().into());
    let voice = &state.voice_settings;
    let settings = app.global::<SettingsData>();
    // Chỉ ghi URL khi hub-core đổi nó, để không đè chữ người dùng đang gõ.
    if url_changed {
        settings.set_ota_url(voice.ota_url.as_str().into());
    }
    settings.set_connection(ConnectionSettings {
        server_status: voice.status_label.as_str().into(),
        device_id: voice.device_id.as_str().into(),
        error: voice.error.as_deref().unwrap_or("").into(),
        enrolled: !voice.enrolling,
        enrollment_code: voice.activation.as_deref().unwrap_or("").into(),
        ..settings.get_connection()
    });
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
