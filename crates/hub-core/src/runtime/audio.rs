use super::effects::Effect;
use crate::{
    AudioCommand, AudioEvent,
    conversation::{Conversation, Input},
};
use jiff::Zoned;
use voice_client::VoiceCommand;

pub(super) fn on(event: AudioEvent, conversation: &mut Conversation, now: Zoned) -> Vec<Effect> {
    match event {
        AudioEvent::WakeWord => conversation.on(Input::WakeWord, now),
        AudioEvent::MicFrame(frame) if conversation.turn_active() => {
            vec![Effect::Voice(VoiceCommand::Audio(frame))]
        }
        _ => Vec::new(),
    }
}

/// Lệnh audio phản ánh cài đặt: âm lượng, wake word (tắt khi tắt mic) và độ nhạy.
fn commands(settings: &crate::DeviceSettings) -> Vec<AudioCommand> {
    vec![
        AudioCommand::SetVolume((settings.volume * 100.0).round() as u8),
        AudioCommand::SetWakeWord {
            enabled: settings.wake_word && !settings.mic_muted,
            sensitivity: settings.sensitivity,
        },
    ]
}

pub(super) fn settings(settings: &crate::DeviceSettings) -> Vec<Effect> {
    commands(settings).into_iter().map(Effect::Audio).collect()
}

/// Lúc khởi động: chỉ gửi khi khác trạng thái mặc định của worker audio.
pub(super) fn initial(settings: &crate::DeviceSettings) -> Vec<Effect> {
    if commands(settings) == commands(&crate::DeviceSettings::default()) {
        return Vec::new();
    }
    self::settings(settings)
}
