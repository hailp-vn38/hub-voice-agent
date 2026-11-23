use crate::{App, ReminderGroup, ReminderItem, ReminderNext, ReminderPast, RemindersData};
use hub_core::present::RemindersView;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

pub(super) fn apply(app: &App, reminders: &RemindersView) {
    let data = app.global::<RemindersData>();
    data.set_title(reminders.title.as_str().into());
    data.set_has_next(reminders.has_next);
    data.set_next(ReminderNext {
        key: reminders.next.key.as_str().into(),
        time: reminders.next.time.as_str().into(),
        until: reminders.next.until.as_str().into(),
        text: reminders.next.text.as_str().into(),
        repeat_text: reminders.next.repeat_text.as_str().into(),
    });
    data.set_groups(ModelRc::from(Rc::new(VecModel::from(
        reminders
            .groups
            .iter()
            .map(|group| ReminderGroup {
                label: group.label.as_str().into(),
                items: ModelRc::from(Rc::new(VecModel::from(
                    group
                        .items
                        .iter()
                        .map(|item| ReminderItem {
                            key: item.key.as_str().into(),
                            time: item.time.as_str().into(),
                            text: item.text.as_str().into(),
                            repeat_text: item.repeat_text.as_str().into(),
                            repeats: item.repeats,
                            fading: false,
                        })
                        .collect::<Vec<_>>(),
                ))),
            })
            .collect::<Vec<_>>(),
    ))));
    data.set_recent(ModelRc::from(Rc::new(VecModel::from(
        reminders
            .recent
            .iter()
            .map(|item| ReminderPast {
                key: item.key.as_str().into(),
                time: item.time.as_str().into(),
                text: item.text.as_str().into(),
                missed: item.missed,
            })
            .collect::<Vec<_>>(),
    ))));
}
