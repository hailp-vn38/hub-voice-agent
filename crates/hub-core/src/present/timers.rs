use crate::{Focus, Timer, UiState};
use jiff::{Span, Zoned};

#[derive(Clone, Debug, PartialEq)]
pub struct TimersView {
    pub items: Vec<TimerCardView>,
    pub title: String,
    pub focus: Option<String>,
    pub serial: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimerCardView {
    pub key: String,
    pub label: String,
    pub remaining: String,
    pub detail: String,
    pub progress: f32,
    pub phase: TimerPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerPhase {
    Running,
    Paused,
    FinalSeconds,
    Finished,
}

pub(super) fn from_state(state: &UiState) -> TimersView {
    let items: Vec<_> = state
        .timers
        .iter()
        .map(|timer| card(timer, &state.now))
        .collect();
    let focus = match state.view.focus.as_ref() {
        Some(Focus::Timer(id)) => Some(id.to_string()),
        _ => None,
    };
    let running = items
        .iter()
        .filter(|timer| matches!(timer.phase, TimerPhase::Running | TimerPhase::FinalSeconds))
        .count();
    TimersView {
        items,
        title: format!("Hẹn giờ · {running} đang chạy"),
        focus,
        serial: state.events.focus,
    }
}

fn card(timer: &Timer, now: &Zoned) -> TimerCardView {
    let remaining = timer
        .paused_seconds
        .unwrap_or_else(|| {
            timer
                .end_seconds
                .saturating_sub(now.timestamp().as_second())
        })
        .max(0);
    let phase = if remaining == 0 {
        TimerPhase::Finished
    } else if timer.paused_seconds.is_some() {
        TimerPhase::Paused
    } else if remaining <= 10 {
        TimerPhase::FinalSeconds
    } else {
        TimerPhase::Running
    };
    let duration = timer.duration_seconds.max(1);
    let progress = (remaining as f32 / duration as f32).clamp(0.0, 1.0);
    let detail = if phase == TimerPhase::Finished {
        format!("Đã hết giờ lúc {}", time_at(now, timer.end_seconds))
    } else if phase == TimerPhase::Paused {
        format!("Tạm dừng · {}", duration_label(timer.duration_seconds))
    } else {
        format!(
            "{} · xong lúc {}",
            duration_label(timer.duration_seconds),
            time_at(now, timer.end_seconds)
        )
    };
    TimerCardView {
        key: timer.id.to_string(),
        label: timer.label.clone(),
        remaining: format!("{}:{:02}", remaining / 60, remaining % 60),
        detail,
        progress,
        phase,
    }
}

fn duration_label(seconds: i64) -> String {
    if seconds < 60 {
        format!("{seconds} giây")
    } else {
        let minutes = seconds / 60 + i64::from(seconds % 60 != 0);
        format!("{minutes} phút")
    }
}

pub(super) fn time_at(now: &Zoned, seconds: i64) -> String {
    let delta = seconds.saturating_sub(now.timestamp().as_second());
    now.checked_add(Span::new().seconds(delta))
        .ok()
        .map(|time| time.strftime("%H:%M").to_string())
        .unwrap_or_default()
}
