use super::{calendar, center, countdowns, home, photos, reminders, timers, voice, weather};
use crate::App;
use crate::audio::Levels;
use hub_core::{HubView, PhotoState, UiState, VoiceSettingsState};
use slint::Weak;
use tokio::sync::watch;

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;

type Update = Box<dyn FnOnce(&App) + Send>;

/// Các lệnh ghi global Slint cho phần của `HubView` đã đổi so với lần hiển thị trước.
struct Changes<'a> {
    previous: Option<&'a HubView>,
    view: &'a HubView,
    updates: Vec<Update>,
}

impl<'a> Changes<'a> {
    fn new(previous: Option<&'a HubView>, view: &'a HubView) -> Self {
        Self {
            previous,
            view,
            updates: Vec::new(),
        }
    }

    fn field<T>(&mut self, get: fn(&HubView) -> &T, apply: fn(&App, &T))
    where
        T: PartialEq + Clone + Send + 'static,
    {
        let current = get(self.view);
        if self
            .previous
            .is_none_or(|previous| get(previous) != current)
        {
            let value = current.clone();
            self.updates.push(Box::new(move |app| apply(app, &value)));
        }
    }

    fn push(&mut self, update: impl FnOnce(&App) + Send + 'static) {
        self.updates.push(Box::new(update));
    }
}

/// Thứ tự giữ như lúc map từng ticket: countdown/reminder trước, rồi Nav và phần còn lại.
fn diff(
    previous: Option<&HubView>,
    view: &HubView,
    voice_settings: Option<(VoiceSettingsState, bool)>,
) -> Vec<Update> {
    let mut changes = Changes::new(previous, view);
    if previous.is_some_and(|previous| agent_photos_finished(previous, view)) {
        changes.push(|_| photos::start_auto_advance());
    }
    changes.field(|view| &view.countdowns, countdowns::update);
    changes.field(|view| &view.reminders, reminders::apply);
    changes.field(|view| &view.nav, |app, nav| voice::apply_nav(app, *nav));
    changes.field(|view| &view.status, voice::apply_status);
    changes.field(|view| &view.voice, voice::apply_voice);
    changes.field(|view| &view.response, voice::apply_response);
    changes.field(|view| &view.tool_trail, voice::apply_tool_trail);
    if let Some((settings, url_changed)) = voice_settings {
        changes.push(move |app| voice::apply_voice_settings(app, &settings, url_changed));
    }
    changes.field(|view| &view.weather, weather::apply);
    changes.field(|view| &view.calendar, calendar::apply);
    changes.field(|view| &view.timers, timers::apply_timers);
    changes.field(
        |view| &view.alert,
        |app, alert| timers::apply_alert(app, alert.as_ref()),
    );
    changes.field(|view| &view.home, home::apply_home);
    changes.field(|view| &view.photos, photos::apply_photos);
    changes.field(|view| &view.center, center::apply_center);
    changes.field(|view| &view.menu, center::apply_menu);
    changes.field(|view| &view.ambient, center::apply_ambient);
    changes.updates
}

fn agent_photos_finished(previous: &HubView, current: &HubView) -> bool {
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
        let mut shown: Option<HubView> = None;
        let mut shown_voice_settings: Option<VoiceSettingsState> = None;
        while rx.changed().await.is_ok() {
            let state = rx.borrow_and_update().clone();
            let view = hub_core::present(&state);
            let voice_settings = (shown_voice_settings.as_ref() != Some(&state.voice_settings))
                .then(|| {
                    let url_changed = shown_voice_settings
                        .as_ref()
                        .is_none_or(|previous| previous.ota_url != state.voice_settings.ota_url);
                    (state.voice_settings.clone(), url_changed)
                });
            let updates = diff(shown.as_ref(), &view, voice_settings);
            shown = Some(view);
            shown_voice_settings = Some(state.voice_settings);
            if updates.is_empty() {
                continue;
            }

            let app = app.clone();
            if slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    for update in updates {
                        update(&app);
                    }
                }
            })
            .is_err()
            {
                return;
            }
        }
    });
}
