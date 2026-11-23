use super::date_format;
use crate::{Reminder, ReminderHistory, ReminderRepeat, UiState};
use jiff::{Span, Timestamp, Zoned, civil::Date};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderItemView {
    pub key: String,
    pub time: String,
    pub text: String,
    pub repeat_text: String,
    pub repeats: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReminderNextView {
    pub key: String,
    pub time: String,
    pub until: String,
    pub text: String,
    pub repeat_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderGroupView {
    pub label: String,
    pub items: Vec<ReminderItemView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderPastView {
    pub key: String,
    pub time: String,
    pub text: String,
    pub missed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemindersView {
    pub title: String,
    pub has_next: bool,
    pub next: ReminderNextView,
    pub groups: Vec<ReminderGroupView>,
    pub recent: Vec<ReminderPastView>,
}

pub(super) fn from_state(state: &UiState) -> RemindersView {
    let mut occurrences: Vec<_> = state
        .reminders
        .iter()
        .filter_map(|reminder| next_at(reminder, &state.now).map(|at| (reminder, at)))
        .collect();
    occurrences.sort_by_key(|(_, at)| at.timestamp());

    let mut groups: BTreeMap<Date, Vec<ReminderItemView>> = BTreeMap::new();
    for (reminder, at) in &occurrences {
        groups
            .entry(at.date())
            .or_default()
            .push(item(reminder, at));
    }
    let groups = groups
        .into_iter()
        .map(|(date, items)| ReminderGroupView {
            label: group_label(date, state.now.date()),
            items,
        })
        .collect();
    let next = occurrences.first().map(|(reminder, at)| ReminderNextView {
        key: reminder.id.to_string(),
        time: at.strftime("%H:%M").to_string(),
        until: until(at.timestamp().as_second() - state.now.timestamp().as_second()),
        text: reminder.label.clone(),
        repeat_text: repeat_text(&reminder.repeat).into(),
    });
    let recent = recent(&state.reminder_history, &state.now);
    let count = occurrences.len();

    RemindersView {
        title: format!("{count} sắp tới"),
        has_next: next.is_some(),
        next: next.unwrap_or_default(),
        groups,
        recent,
    }
}

fn next_at(reminder: &Reminder, now: &Zoned) -> Option<Zoned> {
    let at = reminder.at.parse::<Zoned>().ok()?;
    let oldest_visible = now.checked_sub(Span::new().hours(1)).ok()?;
    if at >= *now || at >= oldest_visible {
        return Some(at);
    }
    reminder
        .next_current(now)
        .and_then(|next| next.parse::<Zoned>().ok())
}

fn item(reminder: &Reminder, at: &Zoned) -> ReminderItemView {
    ReminderItemView {
        key: reminder.id.to_string(),
        time: at.strftime("%H:%M").to_string(),
        text: reminder.label.clone(),
        repeat_text: repeat_text(&reminder.repeat).into(),
        repeats: reminder.repeat != ReminderRepeat::None,
    }
}

fn recent(history: &[ReminderHistory], now: &Zoned) -> Vec<ReminderPastView> {
    let earliest = now.timestamp().as_second().saturating_sub(86_400);
    history
        .iter()
        .filter(|entry| (earliest..=now.timestamp().as_second()).contains(&entry.at_seconds))
        .take(5)
        .filter_map(|entry| {
            let time = Timestamp::from_second(entry.at_seconds)
                .ok()?
                .to_zoned(now.time_zone().clone());
            Some(ReminderPastView {
                key: format!("{}:{}", entry.reminder.id, entry.at_seconds),
                time: time.strftime("%H:%M").to_string(),
                text: entry.reminder.label.clone(),
                missed: entry.missed,
            })
        })
        .collect()
}

fn repeat_text(repeat: &ReminderRepeat) -> &'static str {
    match repeat {
        ReminderRepeat::None => "",
        ReminderRepeat::Daily => "Hằng ngày",
        ReminderRepeat::Weekly => "Hằng tuần",
    }
}

fn group_label(date: Date, today: Date) -> String {
    if date == today {
        return "Hôm nay".into();
    }
    if date == today.checked_add(Span::new().days(1)).unwrap_or(today) {
        return "Ngày mai".into();
    }
    let weekday = date_format::weekday_name(&date);
    if date.year() == today.year() {
        format!("{weekday} {:02}/{:02}", date.day(), date.month())
    } else {
        format!(
            "{weekday} {:02}/{:02}/{}",
            date.day(),
            date.month(),
            date.year()
        )
    }
}

fn until(seconds: i64) -> String {
    if seconds <= 0 {
        return "đến hạn".into();
    }
    if seconds < 3_600 {
        return format!("còn {} phút", (seconds + 59) / 60);
    }
    let hours = seconds / 3_600;
    let minutes = seconds % 3_600 / 60;
    if seconds < 86_400 {
        return if minutes == 0 {
            format!("còn {hours} giờ")
        } else {
            format!("còn {hours} giờ {minutes} phút")
        };
    }
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3_600;
    if hours == 0 {
        format!("còn {days} ngày")
    } else {
        format!("còn {days} ngày {hours} giờ")
    }
}
