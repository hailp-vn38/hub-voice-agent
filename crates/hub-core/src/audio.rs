use voice_client::OpusFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sound {
    Ding,
    TimerChime,
    ReminderChime,
    ToolDone,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sensitivity {
    Low,
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
