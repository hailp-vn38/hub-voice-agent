use crate::{CountdownView, Repeat, UiState, lunar::to_lunar};
use jiff::{Zoned, civil::Date};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountdownUnit {
    Days,
    Live,
    Today,
    Passed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CountdownItemView {
    pub key: String,
    pub label: String,
    pub value: String,
    pub unit: CountdownUnit,
    pub target: String,
    pub lunar: String,
    pub repeat_text: String,
    pub progress: f32,
    pub progress_text: String,
}

impl Default for CountdownItemView {
    fn default() -> Self {
        Self {
            key: String::new(),
            label: String::new(),
            value: String::new(),
            unit: CountdownUnit::Passed,
            target: String::new(),
            lunar: String::new(),
            repeat_text: String::new(),
            progress: 0.0,
            progress_text: "0% chặng đường".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CountdownsView {
    pub title: String,
    pub main: CountdownItemView,
    pub has_main: bool,
    pub others: Vec<CountdownItemView>,
    pub confetti_serial: u64,
}

pub(super) fn from_state(state: &UiState) -> CountdownsView {
    let main_index = state
        .countdowns
        .iter()
        .enumerate()
        .filter(|(_, countdown)| countdown.days >= 0)
        .min_by_key(|(_, countdown)| countdown.days)
        .map(|(index, _)| index);
    let main = main_index.map(|index| card(&state.countdowns[index], &state.now));
    let others = state
        .countdowns
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != main_index)
        .map(|(_, countdown)| card(countdown, &state.now))
        .collect();

    CountdownsView {
        title: format!("Đếm ngược · {} mốc", state.countdowns.len()),
        main: main.clone().unwrap_or_default(),
        has_main: main.is_some(),
        others,
        confetti_serial: state.events.confetti,
    }
}

fn card(countdown: &CountdownView, now: &Zoned) -> CountdownItemView {
    let target: Date = countdown
        .target
        .parse()
        .expect("countdown target is a date");
    let target_at = target.to_zoned(now.time_zone().clone()).ok();
    let unit = if countdown.days < 0 {
        CountdownUnit::Passed
    } else if countdown.days == 0 {
        CountdownUnit::Today
    } else if target_at
        .as_ref()
        .is_some_and(|at| at.timestamp().as_second() - now.timestamp().as_second() < 86_400)
    {
        CountdownUnit::Live
    } else {
        CountdownUnit::Days
    };
    let value = match unit {
        CountdownUnit::Days => countdown.days.to_string(),
        CountdownUnit::Live => target_at
            .as_ref()
            .map(|at| {
                let seconds = at
                    .timestamp()
                    .as_second()
                    .saturating_sub(now.timestamp().as_second())
                    .max(0);
                format!(
                    "{:02}:{:02}:{:02}",
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                )
            })
            .unwrap_or_default(),
        CountdownUnit::Today => format!("Hôm nay là {}", countdown.label),
        CountdownUnit::Passed => "Đã qua".into(),
    };
    let progress = progress(countdown, now, target_at.as_ref());
    let lunar = if countdown.repeat == Repeat::YearlyLunar {
        let lunar = to_lunar(target);
        format!("{}/{} âm lịch", lunar.day, lunar.month)
    } else {
        String::new()
    };

    CountdownItemView {
        key: countdown.id.to_string(),
        label: countdown.label.clone(),
        value,
        unit,
        target: target_label(target, now.date()),
        lunar,
        repeat_text: repeat_text(&countdown.repeat).into(),
        progress,
        progress_text: format!("{:.0}% chặng đường", progress * 100.0),
    }
}

fn progress(countdown: &CountdownView, now: &Zoned, target: Option<&Zoned>) -> f32 {
    let Some(target) = target else {
        return 0.0;
    };
    let Ok(start_date) = countdown.progress_start.parse::<Date>() else {
        return 0.0;
    };
    let Some(mut start) = start_date.to_zoned(now.time_zone().clone()).ok() else {
        return 0.0;
    };
    if let Ok(created_at) = countdown.created_at.parse::<Zoned>()
        && created_at.timestamp() > start.timestamp()
    {
        start = created_at;
    }
    let total = target.timestamp().as_second() - start.timestamp().as_second();
    if total <= 0 {
        return f32::from(now.timestamp() >= target.timestamp());
    }
    let elapsed = now.timestamp().as_second() - start.timestamp().as_second();
    (elapsed as f32 / total as f32).clamp(0.0, 1.0)
}

fn target_label(target: Date, today: Date) -> String {
    let weekday = match target.weekday().to_monday_one_offset() {
        1 => "Thứ Hai",
        2 => "Thứ Ba",
        3 => "Thứ Tư",
        4 => "Thứ Năm",
        5 => "Thứ Sáu",
        6 => "Thứ Bảy",
        _ => "Chủ nhật",
    };
    if target.year() == today.year() {
        format!("{weekday} {:02}/{:02}", target.day(), target.month())
    } else {
        format!(
            "{weekday} {:02}/{:02}/{}",
            target.day(),
            target.month(),
            target.year()
        )
    }
}

fn repeat_text(repeat: &Repeat) -> &'static str {
    match repeat {
        Repeat::None => "",
        Repeat::Yearly => "Hằng năm",
        Repeat::YearlyLunar => "Hằng năm âm lịch",
    }
}
