use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    pub id: i64,
    pub label: String,
    pub duration_seconds: i64,
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
            duration_seconds: duration,
            end_seconds,
            paused_seconds: None,
        });
        Ok(())
    }
    fn find(&self, label: &str) -> Result<usize, String> {
        let labels: Vec<_> = self
            .items
            .iter()
            .map(|timer| timer.label.as_str())
            .collect();
        match crate::selector::select(&labels, label) {
            crate::selector::Selection::NotFound => Err(format!("Timer not found: {label}")),
            crate::selector::Selection::Found(index) => Ok(index),
            crate::selector::Selection::Ambiguous(found) => Err(format!(
                "Ambiguous timer label: {label}; choices: {}",
                found.join(", ")
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
        self.adjust_at(index, delta, paused, now)
    }
    pub fn quick_start(&mut self, minutes: u32, now: i64) -> Result<i64, String> {
        let duration = i64::from(minutes)
            .checked_mul(60)
            .ok_or("Duration too large")?;
        self.start(format!("{minutes} phút"), duration, now)?;
        Ok(self.items.last().expect("start appends a timer").id)
    }
    pub fn toggle_pause(&mut self, id: i64, now: i64) -> Result<(), String> {
        let index = self.find_id(id)?;
        let paused = self.items[index].paused_seconds.is_none();
        self.set_paused(index, paused, now)
    }
    pub fn add_minute(&mut self, id: i64, now: i64) -> Result<(), String> {
        let index = self.find_id(id)?;
        self.adjust_at(index, Some(60), None, now)
    }
    pub fn cancel_id(&mut self, id: i64) -> Result<(), String> {
        let index = self.find_id(id)?;
        self.items.remove(index);
        Ok(())
    }
    fn find_id(&self, id: i64) -> Result<usize, String> {
        self.items
            .iter()
            .position(|timer| timer.id == id)
            .ok_or_else(|| format!("Timer not found: {id}"))
    }
    fn adjust_at(
        &mut self,
        index: usize,
        delta: Option<i64>,
        paused: Option<bool>,
        now: i64,
    ) -> Result<(), String> {
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
        timer.duration_seconds = timer
            .duration_seconds
            .checked_add(delta.unwrap_or(0))
            .ok_or("Duration too large")?
            .max(remaining);
        timer.end_seconds = end;
        timer.paused_seconds = is_paused.then_some(remaining);
        Ok(())
    }
    fn set_paused(&mut self, index: usize, paused: bool, now: i64) -> Result<(), String> {
        let timer = &mut self.items[index];
        let remaining = timer
            .paused_seconds
            .unwrap_or_else(|| timer.end_seconds.saturating_sub(now));
        if remaining <= 0 {
            return Err("Duration must be positive".into());
        }
        timer.end_seconds = now.checked_add(remaining).ok_or("Duration too large")?;
        timer.paused_seconds = paused.then_some(remaining);
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
