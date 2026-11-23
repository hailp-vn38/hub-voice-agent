use tokio::sync::{mpsc, watch};
use voice_client::{ConnectionState, DeviceIdentity, VoiceCommand, VoiceConfig, VoiceEvent};

use super::VoiceSettings;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsStatus {
    Unconfigured,
    Connecting,
    Ready,
    Enrolling,
    Offline { retry_in: std::time::Duration },
}

#[derive(Clone, Debug)]
pub struct SettingsState {
    pub ota_url: String,
    pub device_id: String,
    pub status: SettingsStatus,
    pub activation: Option<String>,
    pub error: Option<String>,
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
        ota_url: settings
            .ota_url()
            .map_or_else(String::new, |url| url.to_string()),
        device_id: settings.device_id().to_owned(),
        status: SettingsStatus::Unconfigured,
        activation: None,
        error: None,
    };
    let (state_tx, state_rx) = watch::channel(state);
    let (commands_tx, commands_rx) = mpsc::unbounded_channel();
    tokio::spawn(run(settings, state_tx, commands_rx));
    (commands_tx, state_rx)
}

async fn run(
    mut settings: VoiceSettings,
    state: watch::Sender<SettingsState>,
    mut commands: mpsc::UnboundedReceiver<SettingsCommand>,
) {
    let device = DeviceIdentity::new(settings.device_id(), settings.client_id());
    let mut voice = settings.ota_url().map(|ota_url| {
        spawn_voice(VoiceConfig {
            ota_url,
            device: device.clone(),
            tools: vec![],
        })
    });
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(SettingsCommand::SaveUrl(input)) => match settings.save_url(&input) {
                    Ok(ota_url) => {
                        let config = VoiceConfig { ota_url: ota_url.clone(), device: device.clone(), tools: vec![] };
                        if let Some((config_tx, _, _)) = &voice { config_tx.send_replace(config); }
                        else { voice = Some(spawn_voice(config)); }
                        state.send_modify(|s| { s.ota_url = ota_url.to_string(); s.error = None; });
                    }
                    Err(error) => state.send_modify(|s| s.error = Some(error.to_string())),
                },
                Some(SettingsCommand::Connect) => {
                    if state.borrow().status == SettingsStatus::Enrolling
                        && let Some((_, voice_tx, _)) = &voice {
                        let _ = voice_tx.send(VoiceCommand::Connect).await;
                    }
                }
                None => break,
            },
            event = async {
                match &mut voice {
                    Some((_, _, events)) => events.recv().await,
                    None => std::future::pending().await,
                }
            } => match event {
                Some(VoiceEvent::Connection(connection)) => state.send_modify(|s| {
                    s.status = match connection {
                        ConnectionState::Connecting => SettingsStatus::Connecting,
                        ConnectionState::Ready => SettingsStatus::Ready,
                        ConnectionState::Enrolling => SettingsStatus::Enrolling,
                        ConnectionState::Offline { retry_in } => SettingsStatus::Offline { retry_in },
                    };
                    s.activation = None;
                }),
                Some(VoiceEvent::Activation { code }) => state.send_modify(|s| s.activation = Some(code)),
                Some(_) => {},
                None => break,
            },
            _ = state.closed() => break,
        }
    }
}

type Voice = (
    watch::Sender<VoiceConfig>,
    mpsc::Sender<VoiceCommand>,
    mpsc::Receiver<VoiceEvent>,
);

fn spawn_voice(config: VoiceConfig) -> Voice {
    let (config_tx, config_rx) = watch::channel(config);
    let (voice_tx, events) = voice_client::spawn(config_rx);
    (config_tx, voice_tx, events)
}
