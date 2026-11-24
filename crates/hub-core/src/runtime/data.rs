use super::{effects::Effect, hub::Hub};
use crate::{Config, DataKind, Store, UserInput};

/// Xoá một loại dữ liệu và trả các effect cập nhật UI.
pub(super) fn clear(
    kind: DataKind,
    hub: &mut Hub,
    store: &mut Store,
    config: &Config,
) -> Vec<Effect> {
    let now = hub.now();
    match kind {
        DataKind::Timers => hub.timers.clear(),
        DataKind::Countdowns => hub.countdowns.clear(now.date()),
        DataKind::Reminders => hub.reminders.clear(),
        DataKind::Photos => {
            let mut effects = Vec::new();
            for id in hub.photos.ids() {
                effects.extend(hub.on_input(UserInput::RemovePhoto { id }, store));
            }
            effects
        }
        DataKind::Cache => {
            if let Err(error) = store.weather_clear() {
                eprintln!("không xoá được cache thời tiết: {error}");
            }
            let mut effects = hub.calendar.reset(config.calendar.clone(), &now);
            effects.extend(hub.weather.update_home(
                &config.weather,
                store,
                now.timestamp().as_second(),
            ));
            effects
        }
    }
}

#[cfg(test)]
#[path = "data_tests.rs"]
mod tests;
