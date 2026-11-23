//! Đồng hồ demo (giờ UTC, đủ cho fixture).
use std::time::{SystemTime, UNIX_EPOCH};

pub fn format_hms(unix_secs: u64) -> String {
    let d = unix_secs % 86_400;
    format!("{:02}:{:02}:{:02}", d / 3600, d % 3600 / 60, d % 60)
}

/// 0 = nửa đêm, 0.5 = trưa.
pub fn day_phase(unix_secs: u64) -> f32 {
    (unix_secs % 86_400) as f32 / 86_400.0
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

pub fn now_hms() -> String {
    format_hms(now_secs())
}

pub fn now_day_phase() -> f32 {
    day_phase(now_secs())
}

#[cfg(test)]
#[path = "clock_tests.rs"]
mod tests;
