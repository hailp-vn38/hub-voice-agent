use jiff::{Span, Zoned};
use voice_client::{ConnectionState, VoiceCommand};

use crate::{AudioCommand, Sound, VoiceState, runtime::effects::Effect};

pub(crate) enum Input {
    Connection(ConnectionState),
    WakeWord,
    Stt(String),
    TtsStart,
    TtsText(String),
    TtsStop,
    Tick,
}

#[derive(Default)]
pub(crate) struct Conversation {
    state: VoiceState,
    deadline: Option<Zoned>,
}

impl Conversation {
    pub(crate) fn turn_active(&self) -> bool {
        matches!(
            self.state,
            VoiceState::Listening | VoiceState::Thinking | VoiceState::Speaking
        )
    }

    pub(crate) fn alert_blocked(&self) -> bool {
        self.turn_active() && self.deadline.is_none()
    }

    pub(crate) fn on(&mut self, input: Input, now: Zoned) -> Vec<Effect> {
        let mut effects = Vec::new();
        let mut stt = None;
        let mut response = None;
        match input {
            Input::Connection(ConnectionState::Ready) if self.state == VoiceState::Offline => {
                self.state = VoiceState::Idle
            }
            Input::Connection(
                ConnectionState::Offline { .. }
                | ConnectionState::Unconfigured
                | ConnectionState::Connecting
                | ConnectionState::Enrolling,
            ) => {
                if self.turn_active() {
                    effects.push(Effect::Audio(AudioCommand::Mic { on: false }));
                }
                self.state = VoiceState::Offline;
                self.deadline = None;
            }
            Input::WakeWord if self.state == VoiceState::Idle => {
                self.state = VoiceState::Listening;
                stt = Some(String::new());
                response = Some(String::new());
                effects.push(Effect::Voice(VoiceCommand::ListenStart));
                effects.push(Effect::Audio(AudioCommand::Play(Sound::Ding)));
                effects.push(Effect::Audio(AudioCommand::Mic { on: true }));
            }
            Input::Stt(text) if self.turn_active() => {
                self.state = VoiceState::Thinking;
                self.deadline = None;
                stt = Some(text);
            }
            Input::TtsStart if self.turn_active() => {
                self.state = VoiceState::Speaking;
                self.deadline = None;
                response = Some(String::new());
            }
            Input::TtsText(text) if self.turn_active() => response = Some(text),
            Input::TtsStop if self.turn_active() => {
                self.state = VoiceState::Listening;
                self.deadline = Some(
                    now.checked_add(Span::new().seconds(10))
                        .expect("valid deadline"),
                );
            }
            Input::Tick
                if self
                    .deadline
                    .as_ref()
                    .is_some_and(|deadline| now >= *deadline) =>
            {
                self.state = VoiceState::Idle;
                self.deadline = None;
                effects.push(Effect::Voice(VoiceCommand::ListenStop));
                effects.push(Effect::Audio(AudioCommand::Mic { on: false }));
            }
            _ => {}
        }
        let state = self.state;
        effects.push(Effect::Ui(Box::new(move |ui| {
            let mut changed = ui.voice.state != state;
            ui.voice.state = state;
            if let Some(text) = stt {
                changed |= ui.voice.stt_text != text;
                ui.voice.stt_text = text;
            }
            if let Some(text) = response {
                changed |= ui.voice.response_text != text;
                ui.voice.response_text = text;
            }
            changed
        })));
        effects
    }
}

#[cfg(test)]
#[path = "conversation_tests.rs"]
mod tests;
