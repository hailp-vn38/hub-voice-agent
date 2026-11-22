use serde::{Deserialize, Serialize};

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
