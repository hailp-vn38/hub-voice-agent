use hub::voice_settings::{SettingsState, SettingsStatus};
use slint::Weak;
use tokio::sync::watch;

use crate::App;

fn apply(app: &App, state: &SettingsState) {
    let status = match state.status {
        SettingsStatus::Unconfigured => "Chưa cấu hình".to_owned(),
        SettingsStatus::Connecting => "Đang kết nối".to_owned(),
        SettingsStatus::Ready => "Đã kết nối".to_owned(),
        SettingsStatus::Enrolling => "Chờ kích hoạt".to_owned(),
        SettingsStatus::Offline { retry_in } => {
            format!("Offline (thử lại sau {} s)", retry_in.as_secs())
        }
    };
    app.set_device_id(state.device_id.as_str().into());
    app.set_voice_status(status.into());
    app.set_activation(state.activation.as_deref().unwrap_or("").into());
    app.set_voice_error(state.error.as_deref().unwrap_or("").into());
    app.set_enrolling(state.status == SettingsStatus::Enrolling);
}

pub fn spawn_voice_bridge(app: Weak<App>, mut rx: watch::Receiver<SettingsState>) {
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
            {
                return;
            }
            if rx.changed().await.is_err() {
                return;
            }
        }
    });
}
