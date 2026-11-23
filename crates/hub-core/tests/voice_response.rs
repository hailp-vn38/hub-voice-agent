mod support;
#[path = "support/voice_response.rs"]
mod voice_response_support;

use hub_core::{AudioCommand, AudioEvent, UserInput, VoiceState, present};
use serde_json::json;
use tokio::sync::watch;
use voice_client::{ConnectionState, TtsOrigin, VoiceCommand, VoiceEvent};
use voice_response_support::{call_tool, connected_hub};

#[tokio::test(start_paused = true)]
async fn wake_stt_and_two_tts_sentences_present_three_speaker_lines() {
    let mut hub = support::Hub::start(
        "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }

    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::ListenStart)
    ));
    hub.audio_rx.recv().await.unwrap();
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Mic { on: true })
    ));
    hub.voice_tx
        .send(VoiceEvent::Stt {
            text: "đặt hẹn giờ".into(),
        })
        .await
        .unwrap();
    wait_for_line(&mut hub.ui_rx, 1).await;
    let response = present(&hub.ui_rx.borrow()).response;
    assert_eq!(response.line_serial, 1);
    assert_eq!(response.speaker, hub_core::ResponseSpeaker::User);
    assert_eq!(
        response
            .words
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>(),
        ["đặt", "hẹn", "giờ"]
    );

    hub.voice_tx
        .send(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent,
        })
        .await
        .unwrap();
    hub.voice_tx
        .send(VoiceEvent::TtsText {
            text: "Đang đếm ngược.".into(),
        })
        .await
        .unwrap();
    wait_for_line(&mut hub.ui_rx, 2).await;
    assert_eq!(
        present(&hub.ui_rx.borrow()).response.speaker,
        hub_core::ResponseSpeaker::Agent
    );
    hub.voice_tx
        .send(VoiceEvent::TtsText {
            text: "Xong sau bảy phút.".into(),
        })
        .await
        .unwrap();
    wait_for_line(&mut hub.ui_rx, 3).await;

    let state = hub.ui_rx.borrow();
    let view = present(&state);
    assert_eq!(state.events.wake, 1);
    assert_eq!(view.voice.state, VoiceState::Speaking);
    assert!(!view.voice.mic_muted);
    assert_eq!(view.response.line_serial, 3);
    assert_eq!(view.response.speaker, hub_core::ResponseSpeaker::Agent);
    assert_eq!(
        view.response
            .words
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>(),
        ["Xong", "sau", "bảy", "phút."]
    );
    assert!(!view.response.fading);
    drop(state);

    hub.voice_tx
        .send(VoiceEvent::TtsStop {
            reason: voice_client::TtsStopReason::Completed,
        })
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Listening {
        hub.ui_rx.changed().await.unwrap();
    }
    tokio::time::advance(std::time::Duration::from_secs(10)).await;
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    assert!(present(&hub.ui_rx.borrow()).response.fading);

    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn orb_tap_starts_a_voice_turn_from_idle() {
    let mut hub = support::Hub::start(
        "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }

    hub.input_tx.send(UserInput::OrbTapped).await.unwrap();

    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::ListenStart)
    ));
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Play(_))
    ));
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Mic { on: true })
    ));
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Listening {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(hub.ui_rx.borrow().events.wake, 1);

    hub.input_tx.send(UserInput::OrbTapped).await.unwrap();
    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::Abort)
    ));
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Mic { on: false })
    ));
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    assert!(present(&hub.ui_rx.borrow()).response.fading);

    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn tool_calls_and_server_tools_update_the_presented_trail() {
    let (mut hub, mut socket, ota_task) = connected_hub().await;

    let success = call_tool(&mut socket, 1, "hub_timer_list", json!({})).await;
    assert!(success["payload"]["result"].is_object(), "{success}");
    let failed = call_tool(&mut socket, 2, "unknown_tool", json!({})).await;
    assert!(failed["payload"]["error"].is_object(), "{failed}");

    let view = present(&hub.ui_rx.borrow());
    assert_eq!(view.tool_trail.steps.len(), 2);
    assert_eq!(view.tool_trail.steps[0].label, "hub_timer_list");
    assert_eq!(
        view.tool_trail.steps[0].state,
        hub_core::ToolTrailState::Done
    );
    assert_eq!(view.tool_trail.steps[1].label, "unknown_tool");
    assert_eq!(
        view.tool_trail.steps[1].state,
        hub_core::ToolTrailState::Failed
    );
    assert_eq!(hub.ui_rx.borrow().events.tool, 2);
    assert_eq!(hub.ui_rx.borrow().events.error, 1);

    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    wait_for_trail(&mut hub.ui_rx, 0).await;
    assert_eq!(hub.ui_rx.borrow().events.wake, 1);

    drop(socket);
    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
    ota_task.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn server_tool_events_move_steps_from_running_to_done_or_failed() {
    let mut hub = support::Hub::start(
        "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }

    hub.voice_tx
        .send(VoiceEvent::ServerTool {
            name: "server_search".into(),
            state: voice_client::ToolState::Started,
        })
        .await
        .unwrap();
    wait_for_trail(&mut hub.ui_rx, 1).await;
    assert_eq!(
        present(&hub.ui_rx.borrow()).tool_trail.steps[0].state,
        hub_core::ToolTrailState::Running
    );
    hub.voice_tx
        .send(VoiceEvent::ServerTool {
            name: "server_search".into(),
            state: voice_client::ToolState::Completed,
        })
        .await
        .unwrap();
    wait_for_step(&mut hub.ui_rx, 0, hub_core::ToolTrailState::Done).await;

    hub.voice_tx
        .send(VoiceEvent::ServerTool {
            name: "server_calendar".into(),
            state: voice_client::ToolState::Started,
        })
        .await
        .unwrap();
    wait_for_trail(&mut hub.ui_rx, 2).await;
    hub.voice_tx
        .send(VoiceEvent::ServerTool {
            name: "server_calendar".into(),
            state: voice_client::ToolState::Failed,
        })
        .await
        .unwrap();
    wait_for_step(&mut hub.ui_rx, 1, hub_core::ToolTrailState::Failed).await;

    let state = hub.ui_rx.borrow();
    assert_eq!(state.events.tool, 2);
    assert_eq!(state.events.error, 1);
    drop(state);
    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}

async fn wait_for_line(ui_rx: &mut watch::Receiver<hub_core::UiState>, serial: u64) {
    while ui_rx.borrow_and_update().voice.response_line_serial != serial {
        ui_rx.changed().await.unwrap();
    }
}

async fn wait_for_trail(ui_rx: &mut watch::Receiver<hub_core::UiState>, len: usize) {
    while ui_rx.borrow_and_update().tool_trail.len() != len {
        ui_rx.changed().await.unwrap();
    }
}

async fn wait_for_step(
    ui_rx: &mut watch::Receiver<hub_core::UiState>,
    index: usize,
    state: hub_core::ToolTrailState,
) {
    while ui_rx.borrow_and_update().tool_trail[index].state != state {
        ui_rx.changed().await.unwrap();
    }
}
