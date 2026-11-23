use jiff::{Span, Zoned, tz::TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReminderRepeat {
    None,
    Daily,
    Weekly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reminder {
    pub id: i64,
    pub label: String,
    pub at: String,
    pub repeat: ReminderRepeat,
    pub weekdays: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReminderDue {
    Alert,
    Missed,
}

/// Parses a reminder time: a zoned time (`…+07:00[Asia/Ho_Chi_Minh]`), an RFC 3339
/// timestamp, or a local date-time without offset, which is read in `tz`.
pub fn parse_at(input: &str, tz: &TimeZone) -> Option<Zoned> {
    let input = input.trim();
    input
        .parse::<Zoned>()
        .ok()
        .or_else(|| {
            input
                .parse::<jiff::Timestamp>()
                .ok()
                .map(|t| t.to_zoned(tz.clone()))
        })
        .or_else(|| {
            input
                .parse::<jiff::civil::DateTime>()
                .ok()?
                .to_zoned(tz.clone())
                .ok()
        })
}

impl Reminder {
    pub fn due(&self, now: Zoned) -> Option<ReminderDue> {
        let at: Zoned = self.at.parse().ok()?;
        let elapsed = now.timestamp().as_second() - at.timestamp().as_second();
        if elapsed < 0 {
            None
        } else if elapsed < 3600 {
            Some(ReminderDue::Alert)
        } else {
            Some(ReminderDue::Missed)
        }
    }

    pub fn next_occurrence(&self) -> Option<String> {
        let at: Zoned = self
            .scheduled_at
            .as_deref()
            .unwrap_or(&self.at)
            .parse()
            .ok()?;
        next_after(&at, &self.repeat, &self.weekdays).map(|next| next.to_string())
    }

    /// The next occurrence that is not already missed at `now`, so a reminder
    /// that missed several days catches up in one step.
    pub fn next_current(&self, now: &Zoned) -> Option<String> {
        let mut next = self.next_occurrence()?.parse::<Zoned>().ok()?;
        let cutoff = now.checked_sub(Span::new().hours(1)).ok()?;
        while next <= cutoff {
            next = next_after(&next, &self.repeat, &self.weekdays)?;
        }
        Some(next.to_string())
    }
}

fn next_after(at: &Zoned, repeat: &ReminderRepeat, weekdays: &[u8]) -> Option<Zoned> {
    let days = match repeat {
        ReminderRepeat::None => return None,
        ReminderRepeat::Daily => 1,
        ReminderRepeat::Weekly => (1..=7).find(|days| {
            at.checked_add(Span::new().days(*days))
                .ok()
                .is_some_and(|next| {
                    weekdays.contains(&(next.weekday().to_monday_one_offset() as u8))
                })
        })?,
    };
    at.checked_add(Span::new().days(days)).ok()
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
