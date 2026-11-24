use crate::audio::Levels;
use crate::{
    App, ConnectionSettings, Response, SettingsData, Speaker, StatusBar, StepState, ToolStep,
    ToolTrail, Voice, VoiceSettingsView, VoiceState, Word,
};
use hub_core::{
    ResponseSpeaker, ToolTrailState, VoiceSettingsState, VoiceState as CoreVoiceState,
    present::{ResponseView, StatusView, ToolTrailView, VoiceView},
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

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

pub(super) fn apply_levels(app: &App, levels: Levels) {
    let voice_levels = app.global::<crate::VoiceLevels>();
    voice_levels.set_mic(levels.mic);
    voice_levels.set_tts(levels.tts);
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

pub(super) fn apply_voice_settings(app: &App, voice: &VoiceSettingsState) {
    let settings = app.global::<SettingsData>();
    settings.set_connection(ConnectionSettings {
        server_status: voice.status_label.as_str().into(),
        device_id: voice.device_id.as_str().into(),
        error: voice.error.as_deref().unwrap_or("").into(),
        enrolled: !voice.enrolling,
        enrollment_code: voice.activation.as_deref().unwrap_or("").into(),
        ..settings.get_connection()
    });
    settings.set_voice(VoiceSettingsView {
        read_reminder: voice.reminder_speech,
        ..settings.get_voice()
    });
}

#[cfg(test)]
#[path = "voice_tests.rs"]
mod tests;
