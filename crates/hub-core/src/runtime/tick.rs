use crate::Clock;

use super::effects::Effect;

pub(super) fn tick(clock: &dyn Clock) -> Vec<Effect> {
    let clock = clock.now().strftime("%H:%M:%S").to_string();
    vec![Effect::Ui(Box::new(move |state| {
        let changed = state.clock != clock;
        if changed {
            state.clock = clock;
        }
        changed
    }))]
}
