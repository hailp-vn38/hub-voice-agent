use crate::{AudioCommand, Reminder, Sound, Timer, runtime::effects::Effect};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Alert {
    Timer(Timer),
    Reminder(Reminder),
}
impl Alert {
    fn sound(&self) -> Sound {
        match self {
            Self::Timer(_) => Sound::TimerChime,
            Self::Reminder(_) => Sound::ReminderChime,
        }
    }
}
#[derive(Default)]
pub(crate) struct Alerts {
    queue: VecDeque<Alert>,
    active: Option<(Alert, i64)>,
}
impl Alerts {
    pub fn active(&self) -> Option<&Alert> {
        self.active.as_ref().map(|(alert, _)| alert)
    }
    pub fn retain_due(&mut self, due: &[Timer], reminders: &[Reminder]) {
        self.queue.retain(|alert| match alert {
            Alert::Timer(timer) => due.iter().any(|item| item.id == timer.id),
            Alert::Reminder(reminder) => reminders.iter().any(|item| item.id == reminder.id),
        });
    }
    pub fn push_timer(&mut self, timer: Timer) {
        self.push_alert(Alert::Timer(timer));
    }
    pub fn push_reminder(&mut self, reminder: Reminder) {
        self.push_alert(Alert::Reminder(reminder));
    }
    fn push_alert(&mut self, alert: Alert) {
        if self.active.as_ref().is_some_and(|(a, _)| *a == alert) || self.queue.contains(&alert) {
            return;
        }
        self.queue.push_back(alert);
    }
    pub fn dismiss(&mut self) -> (Option<Alert>, Vec<Effect>) {
        let Some((alert, _)) = self.active.take() else {
            return (None, Vec::new());
        };
        let sound = alert.sound();
        (
            Some(alert),
            vec![
                Effect::Ui(Box::new(|ui| {
                    ui.alert = None;
                    true
                })),
                Effect::Audio(AudioCommand::StopSound(sound)),
            ],
        )
    }
    pub fn tick(
        &mut self,
        now: i64,
        turn_active: bool,
        synced: bool,
    ) -> (Option<Alert>, Vec<Effect>) {
        let (dismissed, mut effects) = if self.active.as_ref().is_some_and(|(alert, since)| {
            now.saturating_sub(*since)
                >= match alert {
                    Alert::Timer(_) => 120,
                    Alert::Reminder(_) => 60,
                }
        }) {
            self.dismiss()
        } else {
            (None, Vec::new())
        };
        if self.active.is_none()
            && !turn_active
            && synced
            && let Some(alert) = self.queue.pop_front()
        {
            self.active = Some((alert.clone(), now));
            let sound = alert.sound();
            effects.push(Effect::Ui(Box::new(move |ui| {
                ui.alert = Some(alert);
                ui.events.alert = ui.events.alert.saturating_add(1);
                true
            })));
            effects.push(Effect::Audio(AudioCommand::Play(sound)));
        }
        (dismissed, effects)
    }
}
