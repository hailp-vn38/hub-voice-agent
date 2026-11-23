//! Đồng hồ demo (giờ UTC, đủ cho fixture).
use std::time::{SystemTime, UNIX_EPOCH};

pub fn format_hms(unix_secs: u64) -> String {
    let d = unix_secs % 86_400;
    format!("{:02}:{:02}:{:02}", d / 3600, d % 3600 / 60, d % 60)
}

pub fn now_hms() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    format_hms(secs)
}

#[cfg(test)]
#[path = "clock_tests.rs"]
mod tests;
