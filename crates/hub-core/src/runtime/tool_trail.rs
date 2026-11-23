use crate::{ToolTrailState, ToolTrailStep};

use super::effects::Effect;

#[derive(Default)]
pub(super) struct ToolTrail {
    steps: Vec<ToolTrailStep>,
    serial: u64,
}

impl ToolTrail {
    pub(super) fn clear(&mut self) -> Option<Effect> {
        if self.steps.is_empty() {
            return None;
        }
        self.steps.clear();
        Some(self.patch(false, false))
    }

    pub(super) fn start(&mut self, label: String) -> (String, Effect) {
        self.serial = self.serial.saturating_add(1);
        let key = format!("tool-{}", self.serial);
        self.steps.push(ToolTrailStep {
            key: key.clone(),
            label,
            state: ToolTrailState::Running,
        });
        (key, self.patch(true, false))
    }

    pub(super) fn finish(&mut self, key: &str, state: ToolTrailState) -> Option<Effect> {
        let step = self.steps.iter_mut().find(|step| step.key == key)?;
        if step.state != ToolTrailState::Running {
            return None;
        }
        step.state = state;
        Some(self.patch(false, state == ToolTrailState::Failed))
    }

    pub(super) fn finish_latest(&mut self, label: &str, state: ToolTrailState) -> Option<Effect> {
        let key = self
            .steps
            .iter()
            .rfind(|step| step.label == label && step.state == ToolTrailState::Running)?
            .key
            .clone();
        self.finish(&key, state)
    }

    fn patch(&self, started: bool, failed: bool) -> Effect {
        let steps = self.steps.clone();
        Effect::Ui(Box::new(move |ui: &mut crate::UiState| {
            let mut changed = ui.tool_trail != steps;
            ui.tool_trail = steps;
            if started {
                ui.events.tool = ui.events.tool.saturating_add(1);
                changed = true;
            }
            if failed {
                ui.events.error = ui.events.error.saturating_add(1);
                changed = true;
            }
            changed
        }))
    }
}

pub(super) fn outcome(effects: &[Effect]) -> Option<ToolTrailState> {
    effects.iter().find_map(|effect| match effect {
        Effect::Reply(_, result) => Some(if result.is_ok() {
            ToolTrailState::Done
        } else {
            ToolTrailState::Failed
        }),
        Effect::ReplyError(_, _) => Some(ToolTrailState::Failed),
        Effect::PersistPhotos(results, _, Some(_)) => {
            Some(if results.iter().any(|(_, image)| image.is_ok()) {
                ToolTrailState::Done
            } else {
                ToolTrailState::Failed
            })
        }
        _ => None,
    })
}
