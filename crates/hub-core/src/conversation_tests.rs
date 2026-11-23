use super::{Conversation, Input};
use crate::VoiceState;
use std::time::Duration;
use voice_client::ConnectionState;

fn now() -> jiff::Zoned {
    "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
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
    c.on(
        Input::Tick,
        now().checked_add(jiff::Span::new().seconds(9)).unwrap(),
    );
    assert!(c.turn_active());
    c.on(
        Input::Tick,
        now().checked_add(jiff::Span::new().seconds(10)).unwrap(),
    );
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
    c.on(
        Input::Tick,
        now().checked_add(jiff::Span::new().seconds(10)).unwrap(),
    );
    assert_eq!(c.state, VoiceState::Thinking);
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
