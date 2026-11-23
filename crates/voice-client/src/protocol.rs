//! Wire format: hello messages, uplink frames and downlink parsing.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{DirectSpeechError, ToolState, TtsOrigin, TtsStopReason, VoiceEvent};

#[derive(Serialize)]
pub(super) struct ClientHello {
    #[serde(rename = "type")]
    message_type: &'static str,
    version: u8,
    transport: &'static str,
    audio_params: AudioParams,
    features: ClientFeatures,
}

#[derive(Serialize)]
struct AudioParams {
    format: &'static str,
    sample_rate: u32,
    channels: u8,
    frame_duration: u16,
}

#[derive(Serialize)]
struct ClientFeatures {
    mcp: bool,
    tool_status: bool,
    direct_tts: bool,
    aec: bool,
}

impl ClientHello {
    pub(super) fn v1() -> Self {
        Self {
            message_type: "hello",
            version: 1,
            transport: "websocket",
            audio_params: AudioParams {
                format: "opus",
                sample_rate: 16_000,
                channels: 1,
                frame_duration: 60,
            },
            features: ClientFeatures {
                mcp: true,
                tool_status: true,
                direct_tts: true,
                aec: true,
            },
        }
    }
}

#[derive(Deserialize)]
pub(super) struct ServerHello {
    #[serde(rename = "type")]
    message_type: String,
    transport: String,
    session_id: String,
    audio_params: ServerAudioParams,
}

#[derive(Deserialize)]
struct ServerAudioParams {
    format: String,
    sample_rate: u32,
    channels: u8,
    frame_duration: u16,
}

impl ServerHello {
    pub(super) fn is_valid(&self) -> bool {
        self.message_type == "hello"
            && self.transport == "websocket"
            && !self.session_id.is_empty()
            && self.audio_params.format == "opus"
            && self.audio_params.sample_rate == 24_000
            && self.audio_params.channels == 1
            && self.audio_params.frame_duration == 60
    }

    pub(super) fn session_id(&self) -> &str {
        &self.session_id
    }
}

// Uplink frames sent once Ready.

pub(crate) fn listen_start(session_id: &str) -> Value {
    json!({ "type": "listen", "state": "start", "mode": "realtime", "session_id": session_id })
}

pub(crate) fn listen_stop(session_id: &str) -> Value {
    json!({ "type": "listen", "state": "stop", "session_id": session_id })
}

pub(crate) fn abort(session_id: &str) -> Value {
    json!({ "type": "abort", "session_id": session_id })
}

pub(crate) fn direct_speech(session_id: &str, request_id: &str, text: &str) -> Value {
    json!({ "type": "direct_speech", "session_id": session_id, "request_id": request_id, "text": text })
}

pub(crate) fn mcp(session_id: &str, payload: Value) -> Value {
    json!({ "type": "mcp", "session_id": session_id, "payload": payload })
}

// Local direct speech checks; the same limits the server enforces (issue 07).

const MAX_REQUEST_ID_BYTES: usize = 128;
const MAX_TEXT_CHARS: usize = 1_024;
const MAX_TEXT_BYTES: usize = 4_096;

pub(crate) fn validate_direct_speech(
    request_id: &str,
    text: &str,
) -> Result<(), DirectSpeechError> {
    if request_id.is_empty()
        || request_id.len() > MAX_REQUEST_ID_BYTES
        || !request_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.:-".contains(&byte))
    {
        return Err(DirectSpeechError::InvalidRequestId);
    }
    if text.trim().is_empty()
        || text.chars().count() > MAX_TEXT_CHARS
        || text.len() > MAX_TEXT_BYTES
        || text
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
    {
        return Err(DirectSpeechError::InvalidText);
    }
    Ok(())
}

// Downlink text frames other than `mcp`.

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Downlink {
    Stt { text: String },
    Tts(Tts),
    Tool { name: String, state: ToolStateWire },
}

#[derive(Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum Tts {
    Start {
        #[serde(default)]
        source: Source,
        request_id: Option<String>,
    },
    SentenceStart {
        text: String,
    },
    Stop,
    Result {
        #[serde(default)]
        source: Source,
        #[serde(default)]
        outcome: Outcome,
        request_id: Option<String>,
        reason: Option<String>,
    },
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Source {
    ClientDirect,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Rejected,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ToolStateWire {
    Started,
    Completed,
    Failed,
}

/// The public event for a downlink frame, or `None` for frames the client ignores.
pub(crate) fn downlink_event(value: &Value) -> Option<VoiceEvent> {
    let event = match Downlink::deserialize(value).ok()? {
        Downlink::Stt { text } => VoiceEvent::Stt { text },
        Downlink::Tool { name, state } => VoiceEvent::ServerTool {
            name,
            state: match state {
                ToolStateWire::Started => ToolState::Started,
                ToolStateWire::Completed => ToolState::Completed,
                ToolStateWire::Failed => ToolState::Failed,
            },
        },
        Downlink::Tts(Tts::Start { source, request_id }) => VoiceEvent::TtsStart {
            origin: match (source, request_id) {
                (Source::ClientDirect, Some(request_id)) => TtsOrigin::ClientDirect { request_id },
                (Source::ClientDirect, None) => {
                    log::warn!("client_direct tts start without request_id; treating as agent");
                    TtsOrigin::Agent
                }
                (Source::Other, _) => TtsOrigin::Agent,
            },
        },
        Downlink::Tts(Tts::SentenceStart { text }) => VoiceEvent::TtsText { text },
        Downlink::Tts(Tts::Stop) => VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed,
        },
        Downlink::Tts(Tts::Result {
            source: Source::ClientDirect,
            outcome: Outcome::Rejected,
            request_id,
            reason,
        }) => {
            let Some(request_id) = request_id else {
                log::warn!("direct speech rejection without request_id cannot be correlated");
                return None;
            };
            VoiceEvent::DirectSpeechRejected {
                request_id,
                // An empty `Other` reason means the server gave none.
                reason: direct_speech_error(reason.as_deref().unwrap_or("")),
            }
        }
        Downlink::Tts(Tts::Result { .. }) => return None,
    };
    Some(event)
}

fn direct_speech_error(reason: &str) -> DirectSpeechError {
    match reason {
        "unsupported_feature" => DirectSpeechError::UnsupportedFeature,
        "invalid_text" => DirectSpeechError::InvalidText,
        "invalid_request_id" => DirectSpeechError::InvalidRequestId,
        "session_busy" => DirectSpeechError::SessionBusy,
        "rate_limited" => DirectSpeechError::RateLimited,
        "tts_unavailable" => DirectSpeechError::TtsUnavailable,
        "capacity_reached" => DirectSpeechError::CapacityReached,
        "shutting_down" => DirectSpeechError::ShuttingDown,
        other => DirectSpeechError::Other(other.to_owned()),
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
