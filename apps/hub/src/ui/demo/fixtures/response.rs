//! Chữ phản hồi giả: tách từ, đánh dấu phần nhấn, và các kịch bản demo.
use crate::{App, Response, Speaker, StepState, ToolStep, ToolTrail, Word};
use slint::{ComponentHandle, ModelRc, Timer, VecModel};
use std::{rc::Rc, time::Duration};

/// Tách câu thành từ; từ thuộc một cụm trong `emphasis` thì đánh dấu nhấn.
pub fn split_words(text: &str, emphasis: &[&str]) -> Vec<(String, bool)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut flags = vec![false; words.len()];
    for phrase in emphasis {
        let p: Vec<&str> = phrase.split_whitespace().collect();
        if p.is_empty() || p.len() > words.len() {
            continue;
        }
        for i in 0..=words.len() - p.len() {
            if words[i..i + p.len()] == p[..] {
                flags[i..i + p.len()].iter_mut().for_each(|f| *f = true);
            }
        }
    }
    words.into_iter().zip(flags).map(|(w, e)| (w.to_string(), e)).collect()
}

pub fn say(app: &App, speaker: Speaker, text: &str, emphasis: &[&str]) {
    let r = app.global::<Response>();
    let words: Vec<Word> = split_words(text, emphasis)
        .into_iter()
        .map(|(t, e)| Word { text: t.into(), emphasis: e })
        .collect();
    r.set_words(ModelRc::from(Rc::new(VecModel::from(words))));
    r.set_speaker(speaker);
    r.set_fading(false);
    r.set_line_serial(r.get_line_serial() + 1);
}

pub fn user_says(app: &App) {
    say(app, Speaker::User, "Còn bao lâu nữa thì cơm chín?", &[]);
}

/// Ba câu agent nối nhau, mỗi câu một line-serial.
pub fn agent_reply(app: &App) {
    say(app, Speaker::Agent, "Để mình xem nhé.", &[]);
    let weak = app.as_weak();
    Timer::single_shot(Duration::from_millis(1800), move || {
        if let Some(app) = weak.upgrade() {
            say(&app, Speaker::Agent, "Còn 7 phút nữa là cơm chín.", &["7 phút"]);
        }
    });
    let weak = app.as_weak();
    Timer::single_shot(Duration::from_millis(5000), move || {
        if let Some(app) = weak.upgrade() {
            say(&app, Speaker::Agent, "Bạn có muốn mình hẹn giờ giúp không?", &[]);
        }
    });
}

fn set_steps(app: &App, steps: &[(&str, &str, StepState)]) {
    let v: Vec<ToolStep> = steps
        .iter()
        .map(|(k, l, s)| ToolStep { key: (*k).into(), label: (*l).into(), state: *s })
        .collect();
    app.global::<ToolTrail>().set_steps(ModelRc::from(Rc::new(VecModel::from(v))));
}

pub fn tool_server(app: &App) {
    set_steps(app, &[("lich", "Lịch", StepState::Running)]);
    let weak = app.as_weak();
    Timer::single_shot(Duration::from_millis(1500), move || {
        if let Some(app) = weak.upgrade() {
            set_steps(&app, &[("lich", "Lịch", StepState::Done)]);
        }
    });
}

pub fn tool_failed(app: &App) {
    set_steps(app, &[("lich", "Lịch", StepState::Done), ("tin", "Tin tức", StepState::Failed)]);
}

pub fn silence(app: &App) {
    app.global::<Response>().set_fading(true);
    let weak = app.as_weak();
    Timer::single_shot(Duration::from_millis(2500), move || {
        if let Some(app) = weak.upgrade() {
            let r = app.global::<Response>();
            r.set_words(ModelRc::default());
            r.set_fading(false);
            app.global::<ToolTrail>().set_steps(ModelRc::default());
        }
    });
}

#[cfg(test)]
#[path = "response_tests.rs"]
mod tests;
