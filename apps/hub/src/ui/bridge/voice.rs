use crate::{
    App, ConnectionSettings, Response, SettingsData, Speaker, StatusBar, StepState, ToolStep,
    ToolTrail, Voice, VoiceState, Word,
};
use hub_core::{
    ResponseSpeaker, ToolTrailState, UiState, VoiceSettingsState, VoiceState as CoreVoiceState,
    present::{CalendarView, ResponseView, StatusView, ToolTrailView, VoiceView, WeatherDataView},
};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use std::rc::Rc;
use tokio::sync::watch;

pub(super) fn apply_nav(app: &App, screen: hub_core::Screen) {
    app.global::<crate::Nav>()
        .set_screen(super::inputs::to_ui_screen(screen));
}

pub(super) fn apply_status(app: &App, status: &StatusView) {
    let status_bar = app.global::<StatusBar>();
    status_bar.set_time(status.time.as_str().into());
    status_bar.set_offline(status.offline);
}

pub(super) fn apply_voice(app: &App, voice: &VoiceView) {
    let voice_ui = app.global::<Voice>();
    voice_ui.set_state(match voice.state {
        CoreVoiceState::Idle => VoiceState::Idle,
        CoreVoiceState::Listening => VoiceState::Listening,
        CoreVoiceState::Thinking => VoiceState::Thinking,
        CoreVoiceState::Speaking => VoiceState::Speaking,
        CoreVoiceState::Offline => VoiceState::Offline,
    });
    voice_ui.set_mic_muted(voice.mic_muted);
    voice_ui.set_wake_serial(voice.wake_serial as i32);
    voice_ui.set_tool_serial(voice.tool_serial as i32);
    voice_ui.set_error_serial(voice.error_serial as i32);
}

pub(super) fn apply_response(app: &App, response: &ResponseView) {
    let words: Vec<Word> = response
        .words
        .iter()
        .map(|word| Word {
            text: word.text.as_str().into(),
            emphasis: word.emphasis,
        })
        .collect();
    let response_ui = app.global::<Response>();
    response_ui.set_line_serial(response.line_serial as i32);
    response_ui.set_speaker(match response.speaker {
        ResponseSpeaker::User => Speaker::User,
        ResponseSpeaker::Agent => Speaker::Agent,
    });
    response_ui.set_words(ModelRc::from(Rc::new(VecModel::from(words))));
    response_ui.set_fading(response.fading);
}

pub(super) fn apply_tool_trail(app: &App, trail: &ToolTrailView) {
    let steps: Vec<ToolStep> = trail
        .steps
        .iter()
        .map(|step| ToolStep {
            key: step.key.as_str().into(),
            label: step.label.as_str().into(),
            state: match step.state {
                ToolTrailState::Running => StepState::Running,
                ToolTrailState::Done => StepState::Done,
                ToolTrailState::Failed => StepState::Failed,
            },
        })
        .collect();
    app.global::<ToolTrail>()
        .set_steps(ModelRc::from(Rc::new(VecModel::from(steps))));
}

pub(super) fn apply_voice_settings(app: &App, voice: &VoiceSettingsState, url_changed: bool) {
    let settings = app.global::<SettingsData>();
    if url_changed {
        settings.set_ota_url(voice.ota_url.as_str().into());
    }
    settings.set_connection(ConnectionSettings {
        server_status: voice.status_label.as_str().into(),
        device_id: voice.device_id.as_str().into(),
        error: voice.error.as_deref().unwrap_or("").into(),
        enrolled: !voice.enrolling,
        enrollment_code: voice.activation.as_deref().unwrap_or("").into(),
        ..settings.get_connection()
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_changed(
    app: &App,
    nav: Option<hub_core::Screen>,
    status: Option<&StatusView>,
    voice: Option<&VoiceView>,
    response: Option<&ResponseView>,
    tool_trail: Option<&ToolTrailView>,
    voice_settings: Option<(&VoiceSettingsState, bool)>,
    weather: Option<&WeatherDataView>,
    calendar: Option<&CalendarView>,
) {
    if let Some(nav) = nav {
        apply_nav(app, nav);
    }
    if let Some(status) = status {
        apply_status(app, status);
    }
    if let Some(voice) = voice {
        apply_voice(app, voice);
    }
    if let Some(response) = response {
        apply_response(app, response);
    }
    if let Some(tool_trail) = tool_trail {
        apply_tool_trail(app, tool_trail);
    }
    if let Some((settings, url_changed)) = voice_settings {
        apply_voice_settings(app, settings, url_changed);
    }
    if let Some(weather) = weather {
        super::weather::apply(app, weather);
    }
    if let Some(calendar) = calendar {
        super::calendar::apply(app, calendar);
    }
}

pub fn spawn_bridge(app: Weak<App>, mut rx: watch::Receiver<UiState>) {
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
            shown = Some(view);
            shown_voice_settings = Some(state.voice_settings.clone());

            if nav.is_none()
                && status.is_none()
                && voice.is_none()
                && response.is_none()
                && tool_trail.is_none()
                && voice_settings.is_none()
                && weather.is_none()
                && calendar.is_none()
            {
                continue;
            }

            let app = app.clone();
            if slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
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
