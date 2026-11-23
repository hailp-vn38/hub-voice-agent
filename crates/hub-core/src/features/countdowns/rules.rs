use crate::lunar::{LunarDate, julian_day, to_lunar, to_solar};
use jiff::civil::{Date, date};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
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
    #[serde(default)]
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountdownView {
    pub id: i64,
    pub label: String,
    pub target: String,
    pub days: i32,
    pub repeat: Repeat,
    #[serde(skip)]
    pub(crate) created_at: String,
    #[serde(skip)]
    pub(crate) progress_start: String,
}

impl Countdown {
    /// Whether `date` is a real day in the calendar `repeat` names.
    pub fn is_valid(&self) -> bool {
        match self.repeat {
            Repeat::YearlyLunar => self.lunar_date().and_then(to_solar).is_some(),
            _ => self.date.parse::<Date>().is_ok() && !self.lunar_leap,
        }
    }
    fn lunar_date(&self) -> Option<LunarDate> {
        let mut parts = self.date.split('-');
        let lunar = LunarDate {
            year: parts.next()?.parse().ok()?,
            month: parts.next()?.parse().ok()?,
            day: parts.next()?.parse().ok()?,
            leap: self.lunar_leap,
        };
        parts.next().is_none().then_some(lunar)
    }
    pub fn next_occurrence(&self, today: Date) -> Option<Date> {
        if self.repeat == Repeat::YearlyLunar {
            let base = self.lunar_date()?;
            let year = to_lunar(today).year;
            return (year..=year.checked_add(1)?).find_map(|year| {
                let target = lunar_anniversary(LunarDate { year, ..base })?;
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
        Some(CountdownView {
            id: self.id,
            label: self.label.clone(),
            target: target.to_string(),
            days: julian_day(target) - julian_day(today),
            repeat: self.repeat.clone(),
            created_at: self.created_at.clone(),
            progress_start: self.progress_start(target).to_string(),
        })
    }

    fn progress_start(&self, target: Date) -> Date {
        match self.repeat {
            Repeat::None => self
                .created_at
                .parse::<jiff::Zoned>()
                .map(|created| created.date())
                .unwrap_or(target),
            Repeat::Yearly => {
                let Ok(original) = self.date.parse::<Date>() else {
                    return target;
                };
                annual_date(original, target.year() - 1).unwrap_or(target)
            }
            Repeat::YearlyLunar => {
                let Some(base) = self.lunar_date() else {
                    return target;
                };
                let year = to_lunar(target).year - 1;
                lunar_anniversary(LunarDate { year, ..base }).unwrap_or(target)
            }
        }
    }
}

fn annual_date(original: Date, year: i16) -> Option<Date> {
    let day = if original.month() == 2
        && original.day() == 29
        && date(year, 2, 1).days_in_month() == 28
    {
        28
    } else {
        original.day()
    };
    Date::new(year, original.month(), day).ok()
}

/// The solar date of a lunar anniversary in `lunar.year`. A leap month that the
/// year lacks falls back to the regular month, and day 30 of a short month to day 29.
fn lunar_anniversary(lunar: LunarDate) -> Option<Date> {
    let regular = LunarDate {
        leap: false,
        ..lunar
    };
    [lunar, regular]
        .into_iter()
        .flat_map(|candidate| {
            [
                candidate,
                LunarDate {
                    day: 29,
                    ..candidate
                },
            ]
        })
        .filter(|candidate| candidate.day <= lunar.day)
        .find_map(to_solar)
}
