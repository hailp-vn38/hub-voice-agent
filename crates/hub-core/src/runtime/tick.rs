use super::effects::Effect;

/// Publishes the current local time from the actor's clock port.
pub(super) fn tick(now: &jiff::Zoned) -> Vec<Effect> {
    let now = now.clone();
    vec![Effect::Ui(Box::new(move |state| {
        let changed = state.now != now;
        if changed {
            state.now = now;
        }
        changed
    }))]
}
