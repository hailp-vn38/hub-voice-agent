//! Client for the realtime voice service: connection, audio streaming and events.

mod connection;
mod protocol;

use serde_json::Value;
use std::{
    collections::VecDeque,
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot, watch};
use url::Url;

pub use connection::ConnectionState;

const COMMAND_CAPACITY: usize = 64;
const EVENT_CAPACITY: usize = 256;

pub fn spawn(
    config: watch::Receiver<VoiceConfig>,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    spawn_with_jitter(config, random_jitter)
}

fn spawn_with_jitter(
    config: watch::Receiver<VoiceConfig>,
    jitter: impl FnMut(Duration) -> Duration + Send + 'static,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    let (commands_tx, commands_rx) = mpsc::channel(COMMAND_CAPACITY);
    let (events_tx, events_rx) = mpsc::channel(EVENT_CAPACITY);
    tokio::spawn(async move {
        client_loop(config, commands_rx, events_tx, jitter).await;
    });
    (commands_tx, events_rx)
}

async fn client_loop(
    mut config: watch::Receiver<VoiceConfig>,
    mut commands: mpsc::Receiver<VoiceCommand>,
    events: mpsc::Sender<VoiceEvent>,
    mut jitter: impl FnMut(Duration) -> Duration,
) {
    let mut active_config = config.borrow_and_update().clone();
    let mut backoff = Duration::from_secs(1);
    let mut pending_commands = VecDeque::new();
    loop {
        if events.is_closed() {
            return;
        }
        if events
            .send(VoiceEvent::Connection(ConnectionState::Connecting))
            .await
            .is_err()
        {
            return;
        }
        let discovery = connection::await_operation(
            discover_websocket(&active_config),
            &mut config,
            &active_config,
            &mut commands,
            &events,
        )
        .await;
        let connection = match discovery {
            connection::Operation::Complete((url, token)) => {
                connection::connect_and_wait(
                    url,
                    token,
                    active_config.device.clone(),
                    connection::ClientIo {
                        events: &events,
                        commands: &mut commands,
                        pending_commands: &mut pending_commands,
                        config: &mut config,
                        active_config: &mut active_config,
                    },
                )
                .await
            }
            connection::Operation::Reconfigured(updated) => {
                pending_commands.clear();
                active_config = updated;
                backoff = Duration::from_secs(1);
                continue;
            }
            connection::Operation::Stopped => return,
            connection::Operation::Failed => connection::ConnectionEnd::Failed {
                became_ready: false,
            },
        };
        match connection {
            connection::ConnectionEnd::Stopped => return,
            connection::ConnectionEnd::Reconfigured(updated) => {
                pending_commands.clear();
                active_config = updated;
                backoff = Duration::from_secs(1);
                continue;
            }
            connection::ConnectionEnd::Failed { became_ready } => {
                pending_commands.clear();
                if became_ready {
                    backoff = Duration::from_secs(1);
                }
            }
        }
        let retry_in = jitter(backoff);
        if events
            .send(VoiceEvent::Connection(ConnectionState::Offline {
                retry_in,
            }))
            .await
            .is_err()
        {
            return;
        }
        backoff = backoff.saturating_mul(2).min(Duration::from_secs(60));
        match connection::await_operation(
            async {
                tokio::time::sleep(retry_in).await;
                Ok(())
            },
            &mut config,
            &active_config,
            &mut commands,
            &events,
        )
        .await
        {
            connection::Operation::Complete(()) => {}
            connection::Operation::Reconfigured(updated) => {
                active_config = updated;
                backoff = Duration::from_secs(1);
            }
            connection::Operation::Stopped | connection::Operation::Failed => return,
        }
    }
}

fn random_jitter(nominal: Duration) -> Duration {
    static SEED: std::sync::LazyLock<RandomState> = std::sync::LazyLock::new(RandomState::new);
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut hasher = SEED.build_hasher();
    hasher.write_u64(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let percent = (hasher.finish() % 41) as i128 - 20;
    let nanos = nominal.as_nanos() as i128;
    let jittered = (nanos + nanos * percent / 100).max(0) as u128;
    Duration::from_nanos(jittered.min(u64::MAX as u128) as u64)
}

async fn discover_websocket(config: &VoiceConfig) -> Result<(Url, String), ()> {
    let response = reqwest::Client::new()
        .get(config.ota_url.clone())
        .header("device-id", &config.device.device_id)
        .header("client-id", &config.device.client_id)
        .send()
        .await
        .map_err(|_| ())?
        .error_for_status()
        .map_err(|_| ())?;
    let response_body = response.bytes().await.map_err(|_| ())?;
    let body: serde_json::Value = serde_json::from_slice(&response_body).map_err(|_| ())?;
    let websocket = body.get("websocket").ok_or(())?;
    let url = websocket
        .get("url")
        .and_then(serde_json::Value::as_str)
        .ok_or(())?;
    let token = websocket
        .get("token")
        .and_then(serde_json::Value::as_str)
        .ok_or(())?;
    Ok((Url::parse(url).map_err(|_| ())?, token.to_owned()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceConfig {
    pub ota_url: Url,
    pub device: DeviceIdentity,
    pub tools: Vec<ToolDef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub client_id: String,
}

impl DeviceIdentity {
    pub fn new(device_id: impl Into<String>, client_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            client_id: client_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpusFrame(pub Vec<u8>);

#[derive(Debug)]
pub enum VoiceCommand {
    ListenStart,
    ListenStop,
    Abort,
    Audio(OpusFrame),
    DirectSpeech { request_id: String, text: String },
}

#[derive(Debug)]
pub enum VoiceEvent {
    Connection(ConnectionState),
    Activation {
        code: String,
    },
    Stt {
        text: String,
    },
    TtsStart {
        origin: TtsOrigin,
    },
    TtsText {
        text: String,
    },
    TtsAudio(OpusFrame),
    TtsStop {
        reason: TtsStopReason,
    },
    DirectSpeechRejected {
        request_id: String,
        reason: DirectSpeechError,
    },
    ServerTool {
        name: String,
        state: ToolState,
    },
    ToolCall(ToolCall),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TtsOrigin {
    Agent,
    ClientDirect { request_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TtsStopReason {
    Completed,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectSpeechError {
    Offline,
    Rejected(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolState {
    Started,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
    reply: Option<oneshot::Sender<Result<Value, ToolError>>>,
}

impl ToolCall {
    pub fn reply(mut self, result: Result<Value, ToolError>) {
        let _ = self.reply.take().map(|reply| reply.send(result));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod connection_tests;
#[cfg(test)]
#[path = "reconnect_tests/mod.rs"]
mod reconnect_tests;
