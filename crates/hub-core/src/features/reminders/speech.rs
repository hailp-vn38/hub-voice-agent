use crate::{Alert, runtime::effects::Effect};
use voice_client::{ConnectionState, DirectSpeechError, VoiceCommand, VoiceEvent};

#[derive(Default)]
pub struct Speech {
    online: bool,
    current: Option<(i64, String, String, Option<i64>)>,
}
impl Speech {
    pub fn event(&mut self, event: &VoiceEvent, now: i64) {
        match event {
            VoiceEvent::Connection(state) => self.online = matches!(state, ConnectionState::Ready),
            VoiceEvent::DirectSpeechRejected {
                request_id,
                reason: DirectSpeechError::SessionBusy,
            } => {
                if let Some((_, id, _, retry)) = &mut self.current
                    && id == request_id
                {
                    *retry = Some(now + 5);
                }
            }
            _ => {}
        }
    }
    pub fn tick(&mut self, active: Option<&Alert>, now: i64, enabled: bool) -> Vec<Effect> {
        let Some(Alert::Reminder(reminder)) = active else {
            self.current = None;
            return vec![];
        };
        if self
            .current
            .as_ref()
            .is_none_or(|(id, _, _, _)| *id != reminder.id)
        {
            self.current = Some((
                reminder.id,
                uuid::Uuid::new_v4().to_string(),
                reminder.label.clone(),
                Some(now),
            ));
        }
        let Some((_, request_id, text, retry)) = &mut self.current else {
            return vec![];
        };
        if !enabled || !self.online {
            *retry = None;
            return vec![];
        }
        if retry.is_some_and(|when| now >= when) {
            *retry = None;
            return vec![Effect::Voice(VoiceCommand::DirectSpeech {
                request_id: request_id.clone(),
                text: text.clone(),
            })];
        }
        vec![]
    }
}
