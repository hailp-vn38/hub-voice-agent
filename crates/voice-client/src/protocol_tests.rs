use super::downlink_event;
use crate::{DirectSpeechError, ToolState, TtsOrigin, VoiceEvent};

fn parse(text: &str) -> Option<VoiceEvent> {
    downlink_event(&serde_json::from_str(text).unwrap())
}

#[test]
fn rejection_without_reason_still_reaches_the_caller() {
    let event = parse(
        r#"{"type":"tts","state":"result","source":"client_direct","outcome":"rejected","request_id":"r-1"}"#,
    );
    assert!(matches!(
        event,
        Some(VoiceEvent::DirectSpeechRejected { request_id, reason: DirectSpeechError::Other(reason) })
            if request_id == "r-1" && reason.is_empty()
    ));
}

#[test]
fn rejection_without_request_id_cannot_be_correlated() {
    let event = parse(
        r#"{"type":"tts","state":"result","source":"client_direct","outcome":"rejected","reason":"session_busy"}"#,
    );
    assert!(event.is_none());
}

#[test]
fn accepted_or_agent_results_are_ignored() {
    assert!(parse(r#"{"type":"tts","state":"result","source":"client_direct","outcome":"accepted","request_id":"r-1"}"#).is_none());
    assert!(
        parse(r#"{"type":"tts","state":"result","outcome":"rejected","request_id":"r-1"}"#)
            .is_none()
    );
}

#[test]
fn tts_start_origin_follows_source_and_request_id() {
    assert!(matches!(
        parse(r#"{"type":"tts","state":"start","session_id":"s","source":"client_direct","request_id":"r-1"}"#),
        Some(VoiceEvent::TtsStart { origin: TtsOrigin::ClientDirect { request_id } }) if request_id == "r-1"
    ));
    assert!(matches!(
        parse(r#"{"type":"tts","state":"start","source":"client_direct"}"#),
        Some(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent
        })
    ));
    assert!(matches!(
        parse(r#"{"type":"tts","state":"start"}"#),
        Some(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent
        })
    ));
}

#[test]
fn tool_status_and_unknown_frames() {
    assert!(matches!(
        parse(r#"{"type":"tool","name":"search","state":"completed"}"#),
        Some(VoiceEvent::ServerTool { name, state: ToolState::Completed }) if name == "search"
    ));
    assert!(parse(r#"{"type":"tool","name":"search","state":"paused"}"#).is_none());
    assert!(parse(r#"{"type":"tts","state":"later"}"#).is_none());
    assert!(parse(r#"{"type":"mystery"}"#).is_none());
}
