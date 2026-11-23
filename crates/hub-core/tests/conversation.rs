mod support;

use hub_core::{AudioCommand, AudioEvent, Sound};
use support::Hub;
use voice_client::{ConnectionState, VoiceCommand, VoiceEvent};

#[tokio::test(start_paused = true)]
async fn wake_word_starts_conversation() {
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::ListenStart)
    ));
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Play(Sound::Ding))
    ));
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Mic { on: true })
    ));
}

#[tokio::test(start_paused = true)]
async fn offline_wake_and_idle_mic_do_not_send_voice_commands() {
    use voice_client::OpusFrame;
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    hub.audio_tx
        .send(AudioEvent::MicFrame(OpusFrame(vec![1])))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    assert!(hub.voice_rx.try_recv().is_err());
    assert!(hub.audio_rx.try_recv().is_err());
}

#[tokio::test(start_paused = true)]
async fn tts_stop_keeps_mic_on_for_ten_seconds_then_stops() {
    use std::time::Duration;
    use voice_client::{OpusFrame, TtsOrigin, TtsStopReason};
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::ListenStart)
    ));
    hub.audio_rx.recv().await.unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.audio_tx
        .send(AudioEvent::MicFrame(OpusFrame(vec![42])))
        .await
        .unwrap();
    assert!(
        matches!(hub.voice_rx.recv().await, Some(VoiceCommand::Audio(OpusFrame(v))) if v == [42])
    );
    hub.voice_tx
        .send(VoiceEvent::Stt {
            text: "hello".into(),
        })
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Thinking {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(hub.ui_rx.borrow().voice.stt_text, "hello");
    hub.voice_tx
        .send(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent,
        })
        .await
        .unwrap();
    hub.voice_tx
        .send(VoiceEvent::TtsText { text: "hi".into() })
        .await
        .unwrap();
    hub.voice_tx
        .send(VoiceEvent::TtsAudio(OpusFrame(vec![7])))
        .await
        .unwrap();
    assert!(
        matches!(hub.audio_rx.recv().await, Some(AudioCommand::PlayTts(OpusFrame(v))) if v == [7])
    );
    hub.voice_tx
        .send(VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed,
        })
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Listening {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(hub.ui_rx.borrow().voice.response_text, "hi");
    tokio::time::advance(Duration::from_secs(9)).await;
    tokio::task::yield_now().await;
    assert!(hub.voice_rx.try_recv().is_err());
    tokio::time::advance(Duration::from_secs(1)).await;
    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::ListenStop)
    ));
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Mic { on: false })
    ));
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.audio_tx
        .send(AudioEvent::MicFrame(OpusFrame(vec![8])))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    assert!(hub.voice_rx.try_recv().is_err());
}

#[tokio::test(start_paused = true)]
async fn interrupted_tts_stops_playback_and_offline_stops_mic() {
    use std::time::Duration;
    use voice_client::{TtsOrigin, TtsStopReason};
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    hub.voice_rx.recv().await.unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.voice_tx
        .send(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent,
        })
        .await
        .unwrap();
    hub.voice_tx
        .send(VoiceEvent::TtsStop {
            reason: TtsStopReason::Interrupted,
        })
        .await
        .unwrap();
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::StopTts)
    ));
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Offline {
            retry_in: Duration::from_secs(1),
        }))
        .await
        .unwrap();
    assert!(matches!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Mic { on: false })
    ));
    while hub.ui_rx.borrow_and_update().voice.state != hub_core::VoiceState::Offline {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    tokio::task::yield_now().await;
    assert!(hub.voice_rx.try_recv().is_err());
}
