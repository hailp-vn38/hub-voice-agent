//! Client for the realtime voice service: connection, audio streaming and events.

// Ticket 02 connects the public spawn entry point to this transport seam.
#[allow(dead_code)]
mod connection;
#[allow(dead_code)]
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
        let _ = (config, commands_rx, events_tx);
        todo!("OTA discovery is implemented in ticket 02")
    });
    (commands_tx, events_rx)
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
