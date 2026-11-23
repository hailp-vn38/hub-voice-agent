use tokio::sync::{mpsc, watch};
use url::Url;
use voice_client::{ConnectionState, DeviceIdentity, VoiceCommand, VoiceConfig, VoiceEvent};

use super::VoiceSettings;

#[derive(Clone, Debug)]
pub struct SettingsState {
    pub ota_url: String,
    pub device_id: String,
    /// `None` until an OTA URL is saved and voice-client is running.
    pub status: Option<ConnectionState>,
    pub activation: Option<String>,
    pub error: Option<String>,
}

impl SettingsState {
    pub fn status_label(&self) -> String {
        match &self.status {
            None => "Chưa cấu hình".to_owned(),
            Some(ConnectionState::Connecting) => "Đang kết nối".to_owned(),
            Some(ConnectionState::Ready) => "Đã kết nối".to_owned(),
            Some(ConnectionState::Enrolling) => "Chờ kích hoạt".to_owned(),
            Some(ConnectionState::Offline { retry_in }) => {
                format!("Offline (thử lại sau {} s)", retry_in.as_secs())
            }
        }
    }

    pub fn is_enrolling(&self) -> bool {
        self.status == Some(ConnectionState::Enrolling)
    }
}

pub enum SettingsCommand {
    SaveUrl(String),
    Connect,
}

pub fn start(
    settings: VoiceSettings,
) -> (
    mpsc::UnboundedSender<SettingsCommand>,
    watch::Receiver<SettingsState>,
) {
    let state = SettingsState {
        ota_url: settings.ota_url().map(String::from).unwrap_or_default(),
        device_id: settings.device_id().to_owned(),
        status: None,
        activation: None,
        error: None,
    };
    let (state_tx, state_rx) = watch::channel(state);
    let (commands_tx, commands_rx) = mpsc::unbounded_channel();
    tokio::spawn(run(settings, state_tx, commands_rx));
    (commands_tx, state_rx)
}

/// Handles to a running voice-client.
struct Voice {
    config: watch::Sender<VoiceConfig>,
    commands: mpsc::Sender<VoiceCommand>,
    events: mpsc::Receiver<VoiceEvent>,
}

impl Voice {
    fn spawn(config: VoiceConfig) -> Self {
        let (config_tx, config_rx) = watch::channel(config);
        let (commands, events) = voice_client::spawn(config_rx);
        Self {
            config: config_tx,
            commands,
            events,
        }
    }
}

async fn run(
    mut settings: VoiceSettings,
    state: watch::Sender<SettingsState>,
    mut commands: mpsc::UnboundedReceiver<SettingsCommand>,
) {
    let device = DeviceIdentity::new(settings.device_id(), settings.client_id());
    let config = |ota_url: Url| VoiceConfig {
        ota_url,
        device: device.clone(),
        tools: vec![],
    };
    let mut voice = settings.ota_url().map(|url| Voice::spawn(config(url)));
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(SettingsCommand::SaveUrl(input)) => match settings.save_url(&input) {
                    Ok(ota_url) => {
                        let shown = ota_url.to_string();
                        match &voice {
                            Some(voice) => {
                                voice.config.send_replace(config(ota_url));
                            }
                            None => voice = Some(Voice::spawn(config(ota_url))),
                        }
                        state.send_modify(|s| {
                            s.ota_url = shown;
                            s.error = None;
                        });
                    }
                    Err(error) => state.send_modify(|s| s.error = Some(error.to_string())),
                },
                // voice-client decides whether Connect applies (only while
                // Enrolling). `try_send` keeps this loop responsive; a full
                // queue means a Connect is already pending.
                Some(SettingsCommand::Connect) => {
                    if let Some(voice) = &voice {
                        let _ = voice.commands.try_send(VoiceCommand::Connect);
                    }
                }
                None => break,
            },
            event = async {
                match &mut voice {
                    Some(voice) => voice.events.recv().await,
                    None => std::future::pending().await,
                }
            } => match event {
                Some(VoiceEvent::Connection(connection)) => state.send_modify(|s| {
                    s.status = Some(connection);
                    s.activation = None;
                }),
                Some(VoiceEvent::Activation { code }) => {
                    state.send_modify(|s| s.activation = Some(code));
                }
                Some(_) => {}
                // voice-client stopped: drop it so the next save restarts it.
                None => {
                    voice = None;
                    state.send_modify(|s| {
                        s.status = None;
                        s.activation = None;
                        s.error = Some("Voice client đã dừng; lưu lại URL để khởi động lại".into());
                    });
                }
            },
            _ = state.closed() => break,
        }
    }
}
