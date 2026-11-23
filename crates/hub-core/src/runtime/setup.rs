use super::{effects::Effect, hub::Hub};
use crate::{ConfigStore, HomeSettings, Io, Store, setup_web::Update};
use tokio::sync::{oneshot, watch};

/// Applies a Setup web change, then restarts what depends on it.
pub(super) fn apply(
    update: Update,
    reply: oneshot::Sender<Result<(), String>>,
    config: &mut ConfigStore,
    hub: &mut Hub,
    home_settings: &watch::Sender<HomeSettings>,
    io: &Io,
    store: &Store,
) -> Vec<Effect> {
    let calendar_changed =
        matches!(&update, Update::Calendar(value) if *value != config.get().calendar);
    let weather_changed = matches!(&update, Update::Weather(_));
    let home_changed = matches!(&update, Update::Home(_));
    let result = config.update(|c| match update {
        Update::Voice(url) => c.voice.ota_url = Some(url),
        Update::Home(home) => c.home = home,
        Update::Weather(weather) => c.weather = weather,
        Update::Calendar(calendar) => c.calendar = calendar,
    });
    let mut effects = Vec::new();
    if result.is_ok() {
        io.voice_config_tx
            .send_replace(crate::voice_settings::config(config));
        let config = config.get();
        let now = hub.now();
        if calendar_changed {
            effects.extend(hub.calendar.reset(config.calendar.clone(), &now));
        }
        if home_changed {
            home_settings.send_replace(config.home.clone());
            effects.extend(hub.home.settings_changed(&config.home));
        }
        if weather_changed {
            let seconds = now.timestamp().as_second();
            effects.extend(hub.weather.update_home(&config.weather, store, seconds));
        }
        let ota_url = config.voice.ota_url.clone().unwrap_or_default();
        io.ui_tx
            .send_modify(|ui| ui.voice_settings.ota_url = ota_url);
    }
    let _ = reply.send(result.map_err(|e| e.to_string()));
    effects
}
