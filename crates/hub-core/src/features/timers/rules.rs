use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    pub id: i64,
    pub label: String,
    pub end_seconds: i64,
    pub paused_seconds: Option<i64>,
}

#[derive(Default)]
pub struct Timers {
    pub items: Vec<Timer>,
}

impl Timers {
    pub fn start(&mut self, label: String, duration: i64, now: i64) -> Result<(), String> {
        if self.items.len() >= 4 {
            return Err("Maximum of 4 timers".into());
        }
        if duration <= 0 {
            return Err("Duration must be positive".into());
        }
        let end_seconds = now.checked_add(duration).ok_or("Duration too large")?;
        let id = self.items.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        self.items.push(Timer {
            id,
            label,
            end_seconds,
            paused_seconds: None,
        });
        Ok(())
    }
    fn find(&self, label: &str) -> Result<usize, String> {
        let matches: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, t)| t.label.eq_ignore_ascii_case(label))
            .collect();
        match matches.as_slice() {
            [] => Err(format!("Timer not found: {label}")),
            [(index, _)] => Ok(*index),
            _ => Err(format!(
                "Ambiguous timer label: {label}; {} matches",
                matches.len()
            )),
        }
    }
    pub fn adjust(
        &mut self,
        label: &str,
        delta: Option<i64>,
        paused: Option<bool>,
        now: i64,
    ) -> Result<(), String> {
        let index = self.find(label)?;
        let timer = &mut self.items[index];
        let remaining = timer
            .paused_seconds
            .unwrap_or_else(|| timer.end_seconds.saturating_sub(now));
        let remaining = remaining
            .checked_add(delta.unwrap_or(0))
            .ok_or("Duration too large")?;
        if remaining <= 0 {
            return Err("Duration must be positive".into());
        }
        let is_paused = paused.unwrap_or(timer.paused_seconds.is_some());
        let end = now.checked_add(remaining).ok_or("Duration too large")?;
        timer.end_seconds = end;
        timer.paused_seconds = is_paused.then_some(remaining);
        Ok(())
    }
    pub fn cancel(&mut self, label: &str) -> Result<(), String> {
        let index = self.find(label)?;
        self.items.remove(index);
        Ok(())
    }
    pub fn due(&self, now: i64) -> Vec<Timer> {
        self.items
            .iter()
            .filter(|t| t.paused_seconds.is_none() && t.end_seconds <= now)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
