//! Mô phỏng hẹn giờ cho demo (logic thuần, không đụng Slint).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Phase {
    Running,
    Paused,
    FinalSeconds,
    Finished,
}

pub struct Sim {
    pub key: String,
    pub label: String,
    pub total: u32,
    pub remaining: u32,
    pub paused: bool,
    finished_at: Option<u64>,
}

pub fn format_remaining(secs: u32) -> String {
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, secs % 3600 / 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

fn hm(unix_secs: u64) -> String {
    let d = unix_secs % 86_400;
    format!("{:02}:{:02}", d / 3600, d % 3600 / 60)
}

impl Sim {
    pub fn new(key: &str, label: &str, total: u32) -> Self {
        Self { key: key.into(), label: label.into(), total, remaining: total, paused: false, finished_at: None }
    }

    pub fn tick(&mut self) {
        if !self.paused && self.remaining > 0 {
            self.remaining -= 1;
        }
    }

    pub fn add_minute(&mut self) {
        self.remaining += 60;
        self.total += 60;
    }

    pub fn phase(&self) -> Phase {
        if self.remaining == 0 {
            Phase::Finished
        } else if self.paused {
            Phase::Paused
        } else if self.remaining <= 10 {
            Phase::FinalSeconds
        } else {
            Phase::Running
        }
    }

    pub fn progress(&self) -> f32 {
        self.remaining as f32 / self.total.max(1) as f32
    }

    /// Ghi nhận thời điểm hết giờ lần đầu (để hiện "Đã hết giờ lúc").
    pub fn mark_finished(&mut self, now: u64) {
        if self.remaining == 0 && self.finished_at.is_none() {
            self.finished_at = Some(now);
        }
    }

    pub fn detail(&self, now: u64) -> String {
        if self.remaining == 0 {
            format!("Đã hết giờ lúc {}", hm(self.finished_at.unwrap_or(now)))
        } else {
            format!("{} phút · xong lúc {}", self.total.div_ceil(60), hm(now + self.remaining as u64))
        }
    }
}

#[cfg(test)]
#[path = "sim_tests.rs"]
mod tests;
