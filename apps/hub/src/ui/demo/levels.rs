//! Mức tiếng giả cho Orb: sóng sin cộng nhiễu, ~30 Hz.
use crate::{App, Voice, VoiceLevels, VoiceState};
use slint::{ComponentHandle, Timer, TimerMode};
use std::time::{Duration, Instant};

/// Mức 0..1 tại thời điểm `t` giây.
pub fn level(t: f32) -> f32 {
    let wave = 0.5 + 0.35 * (t * 6.0).sin();
    let noise = 0.15 * ((t * 37.0).sin() * (t * 91.0).cos());
    (wave + noise).clamp(0.0, 1.0)
}

pub fn start(app: &App) -> Timer {
    let timer = Timer::default();
    let weak = app.as_weak();
    let t0 = Instant::now();
    timer.start(TimerMode::Repeated, Duration::from_millis(33), move || {
        let Some(app) = weak.upgrade() else { return };
        let v = level(t0.elapsed().as_secs_f32());
        let (mic, tts) = match app.global::<Voice>().get_state() {
            VoiceState::Listening => (v, 0.0),
            VoiceState::Speaking => (0.0, v),
            _ => (0.0, 0.0),
        };
        let l = app.global::<VoiceLevels>();
        l.set_mic(mic);
        l.set_tts(tts);
    });
    timer
}

#[cfg(test)]
#[path = "levels_tests.rs"]
mod tests;
