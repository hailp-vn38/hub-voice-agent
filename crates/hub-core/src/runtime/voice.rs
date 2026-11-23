use super::effects::Effect;
use crate::{
    AudioCommand,
    conversation::{Conversation, Input},
};
use jiff::Zoned;
use voice_client::{TtsStopReason, VoiceEvent};

pub(super) fn on(event: VoiceEvent, conversation: &mut Conversation, now: Zoned) -> Vec<Effect> {
    let mut pending = Vec::new();
    let input = match event {
        VoiceEvent::Connection(state) => Some(Input::Connection(state)),
        VoiceEvent::Stt { text } => Some(Input::Stt(text)),
        VoiceEvent::TtsStart { .. } => Some(Input::TtsStart),
        VoiceEvent::TtsText { text } => Some(Input::TtsText(text)),
        VoiceEvent::TtsAudio(frame) => {
            pending.push(Effect::Audio(AudioCommand::PlayTts(frame)));
            None
        }
        VoiceEvent::TtsStop { reason } => {
            if reason == TtsStopReason::Interrupted {
                pending.push(Effect::Audio(AudioCommand::StopTts));
            }
            Some(Input::TtsStop)
        }
        _ => None,
    };
    if let Some(input) = input {
        pending.extend(conversation.on(input, now));
    }
    pending
}
