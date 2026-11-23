//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod calendar;
mod center;
mod countdowns;
mod home;
mod images;
mod inputs;
mod photos;
mod reminders;
mod timers;
mod voice;
mod weather;

use crate::App;
use crate::audio::Levels;
use hub_core::{
    PhotoState, UiState, VoiceSettingsState,
    present::{
        AlertView, AmbientView, CalendarView, CenterView, HomeView, MenuView, PhotosView,
        ResponseView, StatusView, TimersView, ToolTrailView, VoiceView, WeatherDataView,
    },
};
pub use inputs::connect_inputs;
use slint::Weak;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone, Default)]
pub struct BridgeState {
    countdowns: Arc<Mutex<countdowns::Presentation>>,
}

#[allow(clippy::too_many_arguments)]
fn apply_changed(
    app: &App,
    nav: Option<hub_core::Screen>,
    status: Option<&StatusView>,
    voice: Option<&VoiceView>,
    response: Option<&ResponseView>,
    tool_trail: Option<&ToolTrailView>,
    voice_settings: Option<(&VoiceSettingsState, bool)>,
    weather: Option<&WeatherDataView>,
    calendar: Option<&CalendarView>,
    timers: Option<&TimersView>,
    alert: Option<Option<&AlertView>>,
    home: Option<&HomeView>,
    photos: Option<&PhotosView>,
    center: Option<&CenterView>,
    menu: Option<&MenuView>,
    ambient: Option<&AmbientView>,
) {
    if let Some(nav) = nav {
        voice::apply_nav(app, nav);
    }
    if let Some(status) = status {
        voice::apply_status(app, status);
    }
    if let Some(voice) = voice {
        voice::apply_voice(app, voice);
    }
    if let Some(response) = response {
        voice::apply_response(app, response);
    }
    if let Some(tool_trail) = tool_trail {
        voice::apply_tool_trail(app, tool_trail);
    }
    if let Some((settings, url_changed)) = voice_settings {
        voice::apply_voice_settings(app, settings, url_changed);
    }
    if let Some(weather) = weather {
        weather::apply(app, weather);
    }
    if let Some(calendar) = calendar {
        calendar::apply(app, calendar);
    }
    if let Some(timers) = timers {
        timers::apply_timers(app, timers);
    }
    if let Some(alert) = alert {
        timers::apply_alert(app, alert);
    }
    if let Some(home) = home {
        home::apply_home(app, home);
    }
    if let Some(photos) = photos {
        photos::apply_photos(app, photos);
    }
    if let Some(center) = center {
        center::apply_center(app, center);
    }
    if let Some(menu) = menu {
        center::apply_menu(app, menu);
    }
    if let Some(ambient) = ambient {
        center::apply_ambient(app, ambient);
    }
}

fn agent_photos_finished(previous: &hub_core::HubView, current: &hub_core::HubView) -> bool {
    current.photos.viewing
        && current.photos.viewer.iter().any(|photo| {
            photo.state == PhotoState::Ready
                && previous
                    .photos
                    .photos
                    .iter()
                    .any(|old| old.key == photo.key && old.state == PhotoState::Loading)
        })
}

pub fn spawn_bridge(
    app: Weak<App>,
    mut rx: watch::Receiver<UiState>,
    mut levels_rx: watch::Receiver<Levels>,
    bridge_state: BridgeState,
) {
    let levels_app = app.clone();
    tokio::spawn(async move {
        while levels_rx.changed().await.is_ok() {
            let levels = *levels_rx.borrow_and_update();
            let app = levels_app.clone();
            if slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    voice::apply_levels(&app, levels);
                }
            })
            .is_err()
            {
                return;
            }
        }
    });

    tokio::spawn(async move {
        let mut shown: Option<hub_core::HubView> = None;
        let mut shown_voice_settings: Option<VoiceSettingsState> = None;
        while rx.changed().await.is_ok() {
            let state = rx.borrow_and_update().clone();
            let view = hub_core::present(&state);
            let previous = shown.as_ref();
            let nav =
                (previous.is_none_or(|previous| previous.nav != view.nav)).then_some(view.nav);
            let status = (previous.is_none_or(|previous| previous.status != view.status))
                .then(|| view.status.clone());
            let voice = (previous.is_none_or(|previous| previous.voice != view.voice))
                .then(|| view.voice.clone());
            let response = (previous.is_none_or(|previous| previous.response != view.response))
                .then(|| view.response.clone());
            let tool_trail = (previous
                .is_none_or(|previous| previous.tool_trail != view.tool_trail))
            .then(|| view.tool_trail.clone());
            let countdowns = (previous
                .is_none_or(|previous| previous.countdowns != view.countdowns))
            .then(|| view.countdowns.clone());
            let reminders = (previous.is_none_or(|previous| previous.reminders != view.reminders))
                .then(|| view.reminders.clone());
            let settings_changed = shown_voice_settings.as_ref() != Some(&state.voice_settings);
            let url_changed = shown_voice_settings
                .as_ref()
                .is_none_or(|previous| previous.ota_url != state.voice_settings.ota_url);
            let voice_settings =
                settings_changed.then(|| (state.voice_settings.clone(), url_changed));
            let weather = (previous.is_none_or(|previous| previous.weather != view.weather))
                .then(|| view.weather.clone());
            let calendar = (previous.is_none_or(|previous| previous.calendar != view.calendar))
                .then(|| view.calendar.clone());
            let timers = previous
                .is_none_or(|previous| previous.timers != view.timers)
                .then(|| view.timers.clone());
            let alert = previous
                .is_none_or(|previous| previous.alert != view.alert)
                .then(|| view.alert.clone());
            let home = previous
                .is_none_or(|previous| previous.home != view.home)
                .then(|| view.home.clone());
            let photos = previous
                .is_none_or(|previous| previous.photos != view.photos)
                .then(|| view.photos.clone());
            let center = previous
                .is_none_or(|previous| previous.center != view.center)
                .then(|| view.center.clone());
            let menu = previous
                .is_none_or(|previous| previous.menu != view.menu)
                .then(|| view.menu.clone());
            let ambient = previous
                .is_none_or(|previous| previous.ambient != view.ambient)
                .then(|| view.ambient.clone());
            let start_auto_advance =
                previous.is_some_and(|previous| agent_photos_finished(previous, &view));
            shown = Some(view);
            shown_voice_settings = Some(state.voice_settings.clone());

            if nav.is_none()
                && status.is_none()
                && voice.is_none()
                && response.is_none()
                && tool_trail.is_none()
                && countdowns.is_none()
                && reminders.is_none()
                && voice_settings.is_none()
                && weather.is_none()
                && calendar.is_none()
                && timers.is_none()
                && alert.is_none()
                && home.is_none()
                && photos.is_none()
                && center.is_none()
                && menu.is_none()
                && ambient.is_none()
            {
                continue;
            }

            let app = app.clone();
            let bridge_state = bridge_state.clone();
            if slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    if start_auto_advance {
                        photos::start_auto_advance();
                    }
                    if let Some(countdowns) = countdowns.as_ref() {
                        let display = bridge_state
                            .countdowns
                            .lock()
                            .expect("countdown bridge state poisoned")
                            .update(countdowns);
                        countdowns::apply(&app, &display);
                    }
                    if let Some(reminders) = reminders.as_ref() {
                        reminders::apply(&app, reminders);
                    }
                    apply_changed(
                        &app,
                        nav,
                        status.as_ref(),
                        voice.as_ref(),
                        response.as_ref(),
                        tool_trail.as_ref(),
                        voice_settings
                            .as_ref()
                            .map(|(settings, url_changed)| (settings, *url_changed)),
                        weather.as_ref(),
                        calendar.as_ref(),
                        timers.as_ref(),
                        alert.as_ref().map(|alert| alert.as_ref()),
                        home.as_ref(),
                        photos.as_ref(),
                        center.as_ref(),
                        menu.as_ref(),
                        ambient.as_ref(),
                    );
                }
            })
            .is_err()
            {
                return;
            }
        }
    });
}

#[cfg(test)]
#[path = "timers_tests.rs"]
mod timers_tests;

#[cfg(test)]
#[path = "center_tests.rs"]
mod center_tests;
