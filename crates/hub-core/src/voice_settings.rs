use crate::{ConfigStore, VoiceSettingsState};
use anyhow::{Context, bail};
use url::Url;
use voice_client::{ConnectionState, DeviceIdentity, VoiceConfig};

pub const DEFAULT_OTA_URL: &str = "http://127.0.0.1:8000/voice/ota";
pub(crate) fn parse_url(input: &str) -> anyhow::Result<Url> {
    let url = Url::parse(input.trim()).context("URL không hợp lệ")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        bail!("URL phải dùng HTTP hoặc HTTPS và có tên máy chủ");
    }
    Ok(url)
}
pub(crate) fn config(store: &ConfigStore) -> Option<VoiceConfig> {
    let voice = &store.get().voice;
    Some(VoiceConfig {
        ota_url: parse_url(voice.ota_url.as_deref()?).ok()?,
        device: DeviceIdentity::new(&voice.device_id, &voice.client_id),
        tools: crate::tool_definitions(),
    })
}
pub(crate) fn initial(store: &ConfigStore) -> VoiceSettingsState {
    let voice = &store.get().voice;
    VoiceSettingsState {
        ota_url: voice
            .ota_url
            .clone()
            .unwrap_or_else(|| DEFAULT_OTA_URL.into()),
        device_id: voice.device_id.clone(),
        status_label: "Chưa cấu hình".into(),
        reminder_speech: voice.reminder_speech,
        ..Default::default()
    }
}
pub(crate) fn connection(state: &mut VoiceSettingsState, connection: ConnectionState) {
    state.enrolling = connection == ConnectionState::Enrolling;
    state.status_label = match connection {
        ConnectionState::Unconfigured => "Chưa cấu hình".into(),
        ConnectionState::Connecting => "Đang kết nối".into(),
        ConnectionState::Ready => "Đã kết nối".into(),
        ConnectionState::Enrolling => "Chờ kích hoạt".into(),
        ConnectionState::Offline { retry_in } => {
            format!("Offline (thử lại sau {} s)", retry_in.as_secs())
        }
    };
    state.activation = None;
}
