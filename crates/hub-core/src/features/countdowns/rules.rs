use crate::lunar::{LunarDate, to_lunar, to_solar};
use jiff::civil::{Date, date};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    None,
    Yearly,
    YearlyLunar,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Countdown {
    pub id: i64,
    pub label: String,
    pub date: String,
    pub repeat: Repeat,
    pub lunar_leap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountdownView {
    pub id: i64,
    pub label: String,
    pub target: String,
    pub days: i32,
    pub repeat: Repeat,
}

impl Countdown {
    pub fn next_occurrence(&self, today: Date) -> Option<Date> {
        if self.repeat == Repeat::YearlyLunar {
            let mut parts = self.date.split('-');
            let base = LunarDate {
                year: parts.next()?.parse().ok()?,
                month: parts.next()?.parse().ok()?,
                day: parts.next()?.parse().ok()?,
                leap: self.lunar_leap,
            };
            if parts.next().is_some() {
                return None;
            }
            let year = to_lunar(today).year;
            return (year..=year.checked_add(20)?).find_map(|year| {
                let lunar = LunarDate { year, ..base };
                let target = to_solar(lunar).or_else(|| {
                    (base.day == 30)
                        .then(|| to_solar(LunarDate { day: 29, ..lunar }))
                        .flatten()
                })?;
                (target >= today).then_some(target)
            });
        }
        let original: Date = self.date.parse().ok()?;
        match self.repeat {
            Repeat::None => Some(original),
            Repeat::Yearly => (today.year()..=today.year().checked_add(1)?).find_map(|year| {
                // Leap day falls on February 28 in a common year.
                let day = if original.month() == 2
                    && original.day() == 29
                    && date(year, 2, 1).days_in_month() == 28
                {
                    28
                } else {
                    original.day()
                };
                let target = Date::new(year, original.month(), day).ok()?;
                (target >= today).then_some(target)
            }),
            Repeat::YearlyLunar => unreachable!(),
        }
    }
    pub fn view(&self, today: Date) -> Option<CountdownView> {
        let target = self.next_occurrence(today)?;
        let days = crate::lunar::julian_day(target) - crate::lunar::julian_day(today);
        Some(CountdownView {
            id: self.id,
            label: self.label.clone(),
            target: target.to_string(),
            days,
            repeat: self.repeat.clone(),
        })
    }
}
