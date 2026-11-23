//! Calendar times arrive as RFC 3339 timestamps or all-day `YYYY-MM-DD` dates.
use jiff::{Span, Timestamp, Zoned, civil::Date, tz::TimeZone};

/// The instant a calendar time stands for; an all-day date starts at local midnight.
pub(super) fn instant(value: &str, tz: &TimeZone) -> Option<Timestamp> {
    value.parse::<Timestamp>().ok().or_else(|| {
        let date: Date = value.parse().ok()?;
        Some(date.to_zoned(tz.clone()).ok()?.timestamp())
    })
}

/// Whether `start..end` is a valid range: two timestamps or two all-day dates.
pub(super) fn valid_range(start: &str, end: &str) -> bool {
    let timestamps = (start.parse::<Timestamp>(), end.parse::<Timestamp>());
    let dates = (start.parse::<Date>(), end.parse::<Date>());
    matches!(timestamps, (Ok(a), Ok(b)) if a < b) || matches!(dates, (Ok(a), Ok(b)) if a < b)
}

/// The cached window: 30 days back to 60 days ahead of the local day.
pub(super) fn window(now: &Zoned) -> Option<(Timestamp, Timestamp)> {
    let day = |offset: i64| -> Option<Timestamp> {
        let date = now.date().checked_add(Span::new().days(offset)).ok()?;
        Some(date.to_zoned(now.time_zone().clone()).ok()?.timestamp())
    };
    Some((day(-30)?, day(60)?))
}
