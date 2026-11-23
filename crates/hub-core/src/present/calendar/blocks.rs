#[path = "columns.rs"]
mod columns;

use super::{CalendarBlockView, CalendarKind, PendingState};
use crate::{
    CalendarEvent, Focus, Reminder, ReminderRepeat, UiState,
    ui_state::{CalendarPendingEvent, CalendarPendingState},
};
use jiff::{Span, Timestamp, Zoned, civil::Date, tz::TimeZone};

pub(super) struct CalendarItem {
    pub view: CalendarBlockView,
    pub start: Timestamp,
    pub location: String,
    pub description: String,
}

pub(super) fn for_day(state: &UiState, day: Date) -> Vec<CalendarBlockView> {
    let mut items = items_for_day(state, day);
    items.sort_by_key(|item| (!item.view.all_day, item.start, item.view.key.clone()));

    let timed: Vec<_> = items
        .iter()
        .filter(|item| !item.view.all_day)
        .map(|item| {
            (
                item.view.start_min,
                item.view.end_min.max(item.view.start_min + 30),
            )
        })
        .collect();
    let columns = columns::assign(&timed);
    let mut timed_index = 0;
    items
        .into_iter()
        .map(|mut item| {
            if item.view.all_day {
                item.view.col = 0;
                item.view.cols = 1;
            } else {
                (item.view.col, item.view.cols) = columns[timed_index];
                timed_index += 1;
            }
            item.view
        })
        .collect()
}

pub(super) fn items_for_day(state: &UiState, day: Date) -> Vec<CalendarItem> {
    let timezone = state.now.time_zone();
    state
        .calendar
        .iter()
        .filter_map(|event| {
            let mut item = event_for_day(event, day, &timezone, &state.now)?;
            if let Some(pending) = state
                .calendar_pending
                .iter()
                .find(|pending| pending.key == event.id)
            {
                set_pending(&mut item, pending);
            }
            Some(item)
        })
        .chain(
            state
                .calendar_pending
                .iter()
                .filter(|pending| pending.state != CalendarPendingState::Saved)
                .filter_map(|pending| {
                    let event = CalendarEvent {
                        id: pending.key.clone(),
                        summary: pending.summary.clone(),
                        location: pending.location.clone(),
                        description: None,
                        start: pending.start.clone(),
                        end: pending.end.clone(),
                    };
                    let mut item = event_for_day(&event, day, &timezone, &state.now)?;
                    set_pending(&mut item, pending);
                    Some(item)
                }),
        )
        .chain(
            state
                .reminders
                .iter()
                .filter_map(|reminder| reminder_for_day(reminder, day, &timezone, &state.now)),
        )
        .collect()
}

pub(super) fn selected_day(state: &UiState) -> Date {
    let Some(Focus::Calendar(focus)) = state.view.focus.as_ref() else {
        return state.now.date();
    };
    let timezone = state.now.time_zone();
    if let Some(event) = state.calendar.iter().find(|event| event.id == *focus) {
        if !event.start.contains('T')
            && let Ok(date) = event.start.parse::<Date>()
        {
            return date;
        }
        if let Ok(timestamp) = event.start.parse::<Timestamp>() {
            return timestamp.to_zoned(timezone.clone()).date();
        }
    }
    focus.parse().unwrap_or_else(|_| state.now.date())
}

fn set_pending(item: &mut CalendarItem, pending: &CalendarPendingEvent) {
    item.view.pending = match pending.state {
        CalendarPendingState::Saving => PendingState::Saving,
        CalendarPendingState::Saved => PendingState::Saved,
        CalendarPendingState::Failed => PendingState::Failed,
    };
    item.view.fresh = true;
}

fn event_for_day(
    event: &CalendarEvent,
    day: Date,
    timezone: &TimeZone,
    now: &Zoned,
) -> Option<CalendarItem> {
    if !event.start.contains('T')
        && !event.end.contains('T')
        && let (Ok(start), Ok(end)) = (event.start.parse::<Date>(), event.end.parse::<Date>())
    {
        if start > day || day >= end {
            return None;
        }
        let timestamp = day.to_zoned(timezone.clone()).ok()?.timestamp();
        return Some(CalendarItem {
            view: CalendarBlockView {
                key: event.id.clone(),
                kind: CalendarKind::Event,
                title: event.summary.clone(),
                time: "".into(),
                start_min: 0,
                end_min: 1440,
                all_day: true,
                ongoing: false,
                fresh: false,
                pending: PendingState::None,
                col: 0,
                cols: 1,
            },
            start: timestamp,
            location: event.location.clone().unwrap_or_default(),
            description: event.description.clone().unwrap_or_default(),
        });
    }

    timed_item(
        event.id.clone(),
        CalendarKind::Event,
        event.summary.clone(),
        event.location.clone().unwrap_or_default(),
        event.description.clone().unwrap_or_default(),
        event.start.parse::<Timestamp>().ok()?,
        event.end.parse::<Timestamp>().ok()?,
        day,
        timezone,
        now,
    )
}

fn reminder_for_day(
    reminder: &Reminder,
    day: Date,
    timezone: &TimeZone,
    now: &Zoned,
) -> Option<CalendarItem> {
    let base = reminder.at.parse::<Zoned>().ok().or_else(|| {
        reminder
            .at
            .parse::<Timestamp>()
            .ok()
            .map(|timestamp| timestamp.to_zoned(timezone.clone()))
    })?;
    let days = base.date().until(day).ok()?.get_days();
    if days < 0 {
        return None;
    }
    let occurs = match &reminder.repeat {
        ReminderRepeat::None => days == 0,
        ReminderRepeat::Daily => true,
        ReminderRepeat::Weekly => {
            days == 0
                || reminder
                    .weekdays
                    .contains(&(day.weekday().to_monday_one_offset() as u8))
        }
    };
    if !occurs {
        return None;
    }
    let at = base.checked_add(Span::new().days(i64::from(days))).ok()?;
    let end = at.checked_add(Span::new().minutes(30)).ok()?;
    timed_item(
        format!("reminder-{}", reminder.id),
        CalendarKind::Reminder,
        reminder.label.clone(),
        String::new(),
        String::new(),
        at.timestamp(),
        end.timestamp(),
        day,
        timezone,
        now,
    )
}

fn timed_item(
    key: String,
    kind: CalendarKind,
    title: String,
    location: String,
    description: String,
    start: Timestamp,
    end: Timestamp,
    day: Date,
    timezone: &TimeZone,
    now: &Zoned,
) -> Option<CalendarItem> {
    if start >= end {
        return None;
    }
    let day_start = day.to_zoned(timezone.clone()).ok()?.timestamp();
    let day_end = day
        .checked_add(Span::new().days(1))
        .ok()?
        .to_zoned(timezone.clone())
        .ok()?
        .timestamp();
    if end <= day_start || start >= day_end {
        return None;
    }
    let local_start = start.to_zoned(timezone.clone());
    let local_end = end.to_zoned(timezone.clone());
    let start_min = if start <= day_start {
        0
    } else {
        i32::from(local_start.hour()) * 60 + i32::from(local_start.minute())
    };
    let end_min = if end >= day_end {
        1440
    } else {
        i32::from(local_end.hour()) * 60 + i32::from(local_end.minute())
    };
    let time = format!(
        "{} – {}",
        local_start.strftime("%H:%M"),
        local_end.strftime("%H:%M")
    );
    Some(CalendarItem {
        view: CalendarBlockView {
            key,
            kind,
            title,
            time,
            start_min,
            end_min,
            all_day: false,
            ongoing: start <= now.timestamp() && now.timestamp() < end,
            fresh: false,
            pending: PendingState::None,
            col: 0,
            cols: 1,
        },
        start,
        location,
        description,
    })
}
