use super::{Conversation, Input};
use crate::{VoiceState, runtime::effects::Effect};
use std::time::Duration;
use voice_client::{ConnectionState, VoiceCommand};

fn now() -> jiff::Zoned {
    "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap()
}

fn after(seconds: i64) -> jiff::Zoned {
    now()
        .checked_add(jiff::Span::new().seconds(seconds))
        .unwrap()
}

#[test]
fn transitions_follow_conversation_and_recover_from_offline() {
    let mut c = Conversation::default();
    c.on(Input::WakeWord, now());
    assert_eq!(c.state, VoiceState::Offline);
    c.on(Input::Connection(ConnectionState::Ready), now());
    assert_eq!(c.state, VoiceState::Idle);
    c.on(Input::WakeWord, now());
    assert_eq!(c.state, VoiceState::Listening);
    assert!(c.turn_active());
    c.on(Input::Stt("hello".into()), now());
    assert_eq!(c.state, VoiceState::Thinking);
    c.on(Input::TtsStart, now());
    assert_eq!(c.state, VoiceState::Speaking);
    c.on(Input::TtsStop, now());
    assert_eq!(c.state, VoiceState::Listening);
    c.on(Input::Tick, after(299));
    assert!(c.turn_active());
    c.on(Input::Tick, after(300));
    assert_eq!(c.state, VoiceState::Idle);
    assert!(!c.turn_active());
    c.on(
        Input::Connection(ConnectionState::Offline {
            retry_in: Duration::from_secs(1),
        }),
        now(),
    );
    assert_eq!(c.state, VoiceState::Offline);
}

#[test]
fn new_speech_cancels_silence_deadline() {
    let mut c = Conversation::default();
    c.on(Input::Connection(ConnectionState::Ready), now());
    c.on(Input::WakeWord, now());
    c.on(Input::TtsStop, now());
    c.on(Input::Stt("again".into()), now());
    c.on(Input::Tick, after(300));
    assert_eq!(c.state, VoiceState::Thinking);
}

#[test]
fn listening_without_speech_closes_after_five_minutes() {
    let mut c = Conversation::default();
    c.on(Input::Connection(ConnectionState::Ready), now());
    c.on(Input::WakeWord, now());
    c.on(Input::Tick, after(60));
    assert_eq!(c.state, VoiceState::Listening);
    let effects = c.on(Input::Tick, after(300));
    assert_eq!(c.state, VoiceState::Idle);
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::Voice(VoiceCommand::ListenStop)))
    );
}

#[test]
fn orb_tap_ends_the_conversation_at_once() {
    let mut c = Conversation::default();
    c.on(Input::Connection(ConnectionState::Ready), now());
    c.on(Input::WakeWord, now());
    c.on(Input::TtsStop, now());
    let effects = c.on(Input::OrbTapped, after(1));
    assert_eq!(c.state, VoiceState::Idle);
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::Voice(VoiceCommand::Abort)))
    );
}

#[test]
fn alerts_wait_until_the_agent_has_replied() {
    let mut c = Conversation::default();
    c.on(Input::Connection(ConnectionState::Ready), now());
    c.on(Input::WakeWord, now());
    assert!(c.alert_blocked());
    c.on(Input::TtsStop, now());
    assert!(!c.alert_blocked());
}

#[test]
fn unconfigured_ends_an_active_conversation() {
    let mut c = Conversation::default();
    c.on(Input::Connection(ConnectionState::Ready), now());
    c.on(Input::WakeWord, now());
    c.on(Input::Connection(ConnectionState::Unconfigured), now());
    assert_eq!(c.state, VoiceState::Offline);
    assert!(!c.turn_active());
}

#[test]
fn caption_hides_while_waiting_but_turn_stays_open() {
    let mut c = Conversation::default();
    c.on(Input::Connection(ConnectionState::Ready), now());
    c.on(Input::WakeWord, now());
    c.on(Input::TtsStop, now());
    c.on(Input::Tick, after(9));
    assert!(c.hide_at.is_some());
    let effects = c.on(Input::Tick, after(10));
    assert!(c.hide_at.is_none());
    assert_eq!(c.state, VoiceState::Listening);
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::Voice(_) | Effect::Audio(_)))
    );
}
