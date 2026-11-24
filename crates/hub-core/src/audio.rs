use voice_client::OpusFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sound {
    Ding,
    TimerChime,
    ReminderChime,
    /// Chuông lặp dùng mẫu "Ping", khi người dùng chọn Ping cho hẹn giờ hoặc Reminder.
    PingChime,
    /// Nghe thử một lần ở màn Setting.
    Preview(Tone),
    ToolDone,
}

/// Mẫu âm có sẵn để chọn làm chuông.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Ping,
    Reminder,
}

/// Tên các mẫu âm hiện trên màn Setting.
pub const SOUND_NAMES: [&str; 2] = ["Ping", "Reminder"];

impl Sound {
    /// Âm nghe thử cho tên trong `SOUND_NAMES`.
    pub fn preview(name: &str) -> Option<Self> {
        match name {
            "Ping" => Some(Self::Preview(Tone::Ping)),
            "Reminder" => Some(Self::Preview(Tone::Reminder)),
            _ => None,
        }
    }

    /// Âm giao diện bị tắt bởi "Âm giao diện"; chuông báo và nghe thử thì không.
    pub(crate) fn is_ui(&self) -> bool {
        matches!(self, Self::Ding | Self::ToolDone)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sensitivity {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AudioCommand {
    Mic {
        on: bool,
    },
    PlayTts(OpusFrame),
    StopTts,
    Play(Sound),
    StopSound(Sound),
    SetVolume(u8),
    SetWakeWord {
        enabled: bool,
        sensitivity: Sensitivity,
    },
}

#[derive(Debug)]
pub enum AudioEvent {
    WakeWord,
    MicFrame(OpusFrame),
    TtsPlaybackFinished,
}
