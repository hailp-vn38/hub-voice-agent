use crate::{AudioCommand, Sound, Timer, runtime::effects::Effect};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Alert {
    Timer(Timer),
}

#[derive(Default)]
pub(crate) struct Alerts {
    queue: VecDeque<Timer>,
    active: Option<(Timer, i64)>,
}

impl Alerts {
    pub fn retain_due(&mut self, due: &[Timer]) {
        self.queue
            .retain(|timer| due.iter().any(|item| item.id == timer.id));
    }
    pub fn push(&mut self, timer: Timer) {
        if self.active.as_ref().is_some_and(|(t, _)| t.id == timer.id)
            || self.queue.iter().any(|t| t.id == timer.id)
        {
            return;
        }
        self.queue.push_back(timer);
    }
    pub fn dismiss(&mut self) -> (Option<i64>, Vec<Effect>) {
        let Some((timer, _)) = self.active.take() else {
            return (None, Vec::new());
        };
        (
            Some(timer.id),
            vec![
                Effect::Ui(Box::new(|ui| {
                    ui.alert = None;
                    true
                })),
                Effect::Audio(AudioCommand::StopSound(Sound::TimerChime)),
            ],
        )
    }
    pub fn tick(
        &mut self,
        now: i64,
        turn_active: bool,
        synced: bool,
    ) -> (Option<i64>, Vec<Effect>) {
        let (dismissed, mut effects) = if self
            .active
            .as_ref()
            .is_some_and(|(_, since)| now.saturating_sub(*since) >= 120)
        {
            self.dismiss()
        } else {
            (None, Vec::new())
        };
        if self.active.is_none() && !turn_active && synced {
            if let Some(timer) = self.queue.pop_front() {
                self.active = Some((timer.clone(), now));
                effects.push(Effect::Ui(Box::new(move |ui| {
                    ui.alert = Some(Alert::Timer(timer));
                    true
                })));
                effects.push(Effect::Audio(AudioCommand::Play(Sound::TimerChime)));
            }
        }
        (dismissed, effects)
    }
}
