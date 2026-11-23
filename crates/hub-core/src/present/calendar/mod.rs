mod blocks;
mod upcoming;

use crate::UiState;
use jiff::civil::Date;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarKind {
    Event,
    Reminder,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PendingState {
    #[default]
    None,
    Saving,
    Saved,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDayView {
    pub key: String,
    pub weekday: String,
    pub day: String,
    pub dots: i32,
    pub today: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarBlockView {
    pub key: String,
    pub kind: CalendarKind,
    pub title: String,
    pub time: String,
    pub start_min: i32,
    pub end_min: i32,
    pub all_day: bool,
    pub ongoing: bool,
    pub fresh: bool,
    pub pending: PendingState,
    pub col: i32,
    pub cols: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarUpcomingView {
    pub key: String,
    pub date: String,
    pub time: String,
    pub title: String,
    pub more: i32,
    pub fresh: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarMonthDayView {
    pub key: String,
    pub day: String,
    pub dots: i32,
    pub in_month: bool,
    pub today: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDetailView {
    pub key: String,
    pub title: String,
    pub time: String,
    pub location: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarView {
    pub month_title: String,
    pub synced: String,
    pub stale: bool,
    pub week: Vec<CalendarDayView>,
    pub selected_day: String,
    pub blocks: Vec<CalendarBlockView>,
    pub now_min: i32,
    pub upcoming: Vec<CalendarUpcomingView>,
    pub month: Vec<CalendarMonthDayView>,
    pub details: Vec<CalendarDetailView>,
    pub focus: String,
    pub focus_serial: u64,
}

pub(super) fn from_state(state: &UiState) -> CalendarView {
    let selected = blocks::selected_day(state);
    let synced = state
        .calendar_last_sync
        .and_then(|seconds| jiff::Timestamp::from_second(seconds).ok())
        .map(|time| {
            time.to_zoned(state.now.time_zone().clone())
                .strftime("%H:%M")
                .to_string()
        })
        .unwrap_or_default();
    let stale = state.calendar_online != Some(true)
        || state
            .calendar_last_sync
            .is_none_or(|sync| state.now.timestamp().as_second().saturating_sub(sync) > 5 * 60);
    let focus = match state.view.focus.as_ref() {
        Some(crate::Focus::Calendar(focus)) => focus.clone(),
        _ => String::new(),
    };
    let details = blocks::items_for_day(state, selected)
        .into_iter()
        .map(|item| CalendarDetailView {
            key: item.view.key,
            title: item.view.title,
            time: if item.view.all_day {
                "Cả ngày".into()
            } else {
                item.view.time
            },
            location: item.location,
            description: item.description,
        })
        .collect();
    CalendarView {
        month_title: format!("Tháng {}", selected.month()),
        synced,
        stale,
        week: week(state, state.now.date()),
        selected_day: selected.to_string(),
        blocks: blocks::for_day(state, selected),
        now_min: i32::from(state.now.hour()) * 60 + i32::from(state.now.minute()),
        upcoming: upcoming::from_state(state, selected),
        month: month(state, selected),
        details,
        focus,
        focus_serial: state.events.focus,
    }
}

pub(super) fn center_today(state: &UiState) -> Vec<super::CenterEvent> {
    blocks::for_day(state, state.now.date())
        .into_iter()
        .filter(|event| event.kind == CalendarKind::Event)
        .map(|event| super::CenterEvent {
            key: event.key,
            time: if event.all_day {
                "Cả ngày".into()
            } else {
                event.time
            },
            title: event.title,
            now: event.ongoing,
        })
        .collect()
}

fn week(state: &UiState, today: Date) -> Vec<CalendarDayView> {
    (0..7)
        .filter_map(|offset| {
            let date = today.checked_add(jiff::Span::new().days(offset)).ok()?;
            let dots = blocks::items_for_day(state, date)
                .iter()
                .filter(|item| item.view.pending != PendingState::Failed)
                .count()
                .min(3) as i32;
            Some(CalendarDayView {
                key: date.to_string(),
                weekday: weekday(date),
                day: date.day().to_string(),
                dots,
                today: date == today,
            })
        })
        .collect()
}

fn month(state: &UiState, selected: Date) -> Vec<CalendarMonthDayView> {
    let Some(first) = Date::new(selected.year(), selected.month(), 1).ok() else {
        return Vec::new();
    };
    let lead = i64::from(first.weekday().to_monday_one_offset() - 1);
    let Some(start) = first.checked_sub(jiff::Span::new().days(lead)).ok() else {
        return Vec::new();
    };
    (0..42)
        .filter_map(|offset| {
            let date = start.checked_add(jiff::Span::new().days(offset)).ok()?;
            let dots = blocks::items_for_day(state, date)
                .iter()
                .filter(|item| item.view.pending != PendingState::Failed)
                .count()
                .min(3) as i32;
            Some(CalendarMonthDayView {
                key: date.to_string(),
                day: date.day().to_string(),
                dots,
                in_month: date.month() == selected.month() && date.year() == selected.year(),
                today: date == state.now.date(),
            })
        })
        .collect()
}

fn weekday(date: Date) -> String {
    match date.weekday().to_monday_one_offset() {
        1 => "T2",
        2 => "T3",
        3 => "T4",
        4 => "T5",
        5 => "T6",
        6 => "T7",
        _ => "CN",
    }
    .into()
}
