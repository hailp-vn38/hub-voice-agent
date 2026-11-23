use super::CalendarUpcomingView;
use crate::{CalendarEvent, UiState, ui_state::CalendarPendingState};
use jiff::{Timestamp, civil::Date};
use std::collections::BTreeMap;

struct UpcomingItem {
    key: String,
    title: String,
    time: String,
    fresh: bool,
    at: Timestamp,
}

pub(super) fn from_state(state: &UiState, selected: Date) -> Vec<CalendarUpcomingView> {
    let timezone = state.now.time_zone();
    let mut by_date: BTreeMap<Date, Vec<UpcomingItem>> = BTreeMap::new();
    for event in &state.calendar {
        let fresh = state
            .calendar_pending
            .iter()
            .any(|pending| pending.key == event.id && pending.state == CalendarPendingState::Saved);
        if let Some((date, item)) = item(event, &timezone, fresh)
            && date > selected
        {
            by_date.entry(date).or_default().push(item);
        }
    }
    for pending in state
        .calendar_pending
        .iter()
        .filter(|pending| pending.state == CalendarPendingState::Saving)
    {
        let event = CalendarEvent {
            id: pending.key.clone(),
            summary: pending.summary.clone(),
            location: pending.location.clone(),
            description: None,
            start: pending.start.clone(),
            end: pending.end.clone(),
        };
        if let Some((date, item)) = item(&event, &timezone, true)
            && date > selected
        {
            by_date.entry(date).or_default().push(item);
        }
    }

    by_date
        .into_iter()
        .take(5)
        .filter_map(|(date, mut items)| {
            items.sort_by_key(|item| (item.at, item.key.clone()));
            let first = items.first()?;
            Some(CalendarUpcomingView {
                key: first.key.clone(),
                date: format!("{}/{}", date.day(), date.month()),
                time: first.time.clone(),
                title: first.title.clone(),
                more: items.len().saturating_sub(1) as i32,
                fresh: first.fresh,
            })
        })
        .collect()
}

fn item(
    event: &CalendarEvent,
    timezone: &jiff::tz::TimeZone,
    fresh: bool,
) -> Option<(Date, UpcomingItem)> {
    if !event.start.contains('T')
        && let Ok(date) = event.start.parse::<Date>()
    {
        let at = date.to_zoned(timezone.clone()).ok()?.timestamp();
        return Some((
            date,
            UpcomingItem {
                key: event.id.clone(),
                title: event.summary.clone(),
                time: String::new(),
                fresh,
                at,
            },
        ));
    }
    let at = event.start.parse::<Timestamp>().ok()?;
    let local = at.to_zoned(timezone.clone());
    Some((
        local.date(),
        UpcomingItem {
            key: event.id.clone(),
            title: event.summary.clone(),
            time: local.strftime("%H:%M").to_string(),
            fresh,
            at,
        },
    ))
}
