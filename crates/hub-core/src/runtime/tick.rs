use super::effects::Effect;

/// Publishes the local clock as `HH:MM:SS`.
pub(super) fn tick(now: &jiff::Zoned) -> Vec<Effect> {
    let clock = now.strftime("%H:%M:%S").to_string();
    vec![Effect::Ui(Box::new(move |state| {
        let changed = state.clock != clock;
        if changed {
            state.clock = clock;
        }
        changed
    }))]
}
