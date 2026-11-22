//! Client for the realtime voice service: connection, audio streaming and events.

mod connection;
mod protocol;

use serde_json::Value;
use tokio::sync::{mpsc, oneshot, watch};
use url::Url;

pub use connection::ConnectionState;

const COMMAND_CAPACITY: usize = 64;
const EVENT_CAPACITY: usize = 256;

pub fn spawn(
    config: watch::Receiver<VoiceConfig>,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    let (commands_tx, commands_rx) = mpsc::channel(COMMAND_CAPACITY);
    let (events_tx, events_rx) = mpsc::channel(EVENT_CAPACITY);
    tokio::spawn(async move {
        let mut commands_rx = commands_rx;
        let _ = events_tx
            .send(VoiceEvent::Connection(ConnectionState::Connecting))
            .await;
        let config = config.borrow().clone();
        let result = async {
            let (url, token) = discover_websocket(&config).await?;
            connection::connect_and_wait(url, token, config.device, &events_tx, &mut commands_rx)
                .await
        }
        .await;
        if result.is_err() {
            let _ = events_tx
                .send(VoiceEvent::Connection(ConnectionState::Offline {
                    retry_in: std::time::Duration::from_secs(1),
                }))
                .await;
        }
    });
    (commands_tx, events_rx)
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
