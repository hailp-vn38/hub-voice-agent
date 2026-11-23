use super::effects::Effect;
use crate::{
    AudioEvent,
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
