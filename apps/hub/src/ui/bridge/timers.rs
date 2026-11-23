use crate::{AlertData, AlertKind, App, TimerCard, TimerPhase, TimersData};
use hub_core::present::{
    AlertKind as CoreAlertKind, AlertView, TimerPhase as CoreTimerPhase, TimersView,
};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;

thread_local! {
    static TIMER_CARDS: Rc<VecModel<TimerCard>> = Rc::new(VecModel::default());
}

pub(super) fn apply_timers(app: &App, timers: &TimersView) {
    let data = app.global::<TimersData>();
    data.set_title(timers.title.as_str().into());
    data.set_focus(timers.focus.as_deref().unwrap_or("").into());
    data.set_focus_serial(timers.serial as i32);

    TIMER_CARDS.with(|cards| {
        let mut row = 0;
        while row < cards.row_count() {
            let stale = cards.row_data(row).is_some_and(|card| {
                timers
                    .items
                    .iter()
                    .all(|item| item.key != card.key.as_str())
            });
            if stale {
                cards.remove(row);
            } else {
                row += 1;
            }
        }

        for (row, item) in timers.items.iter().enumerate() {
            let card = TimerCard {
                key: item.key.as_str().into(),
                label: item.label.as_str().into(),
                remaining: item.remaining.as_str().into(),
                detail: item.detail.as_str().into(),
                progress: item.progress,
                phase: match item.phase {
                    CoreTimerPhase::Running => TimerPhase::Running,
                    CoreTimerPhase::Paused => TimerPhase::Paused,
                    CoreTimerPhase::FinalSeconds => TimerPhase::FinalSeconds,
                    CoreTimerPhase::Finished => TimerPhase::Finished,
                },
            };
            if row < cards.row_count() {
                cards.set_row_data(row, card);
            } else {
                cards.push(card);
            }
        }
        data.set_timers(ModelRc::from(cards.clone()));
    });
}

pub(super) fn apply_alert(app: &App, alert: Option<&AlertView>) {
    let data = app.global::<AlertData>();
    let Some(alert) = alert else {
        data.set_kind(AlertKind::None);
        return;
    };

    data.set_title(alert.title.as_str().into());
    data.set_time(alert.time.as_str().into());
    data.set_body(alert.body.as_str().into());
    data.set_serial(alert.serial as i32);
    data.set_kind(match alert.kind {
        CoreAlertKind::Timer => AlertKind::Timer,
        CoreAlertKind::Reminder => AlertKind::Reminder,
    });
}
