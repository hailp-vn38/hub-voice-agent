use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    None,
    Daily,
    Weekly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reminder {
    pub id: i64,
    pub label: String,
    pub at: String,
    pub repeat: Repeat,
    pub weekdays: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Due {
    Alert,
    Missed,
}

impl Reminder {
    pub fn due(&self, now: jiff::Zoned) -> Option<Due> {
        let at: jiff::Zoned = self.at.parse().ok()?;
        let elapsed = now.timestamp().as_second() - at.timestamp().as_second();
        if elapsed < 0 {
            None
        } else if elapsed < 3600 {
            Some(Due::Alert)
        } else {
            Some(Due::Missed)
        }
    }

    pub fn next_occurrence(&self) -> Option<String> {
        let at: jiff::Zoned = self
            .scheduled_at
            .as_deref()
            .unwrap_or(&self.at)
            .parse()
            .ok()?;
        let days = match self.repeat {
            Repeat::None => return None,
            Repeat::Daily => 1,
            Repeat::Weekly => (1..=7).find(|days| {
                at.checked_add(jiff::Span::new().days(*days))
                    .ok()
                    .is_some_and(|next| {
                        self.weekdays
                            .contains(&(next.weekday().to_monday_one_offset() as u8))
                    })
            })?,
        };
        at.checked_add(jiff::Span::new().days(days))
            .ok()
            .map(|at| at.to_string())
    }
}
