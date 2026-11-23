#[allow(dead_code)]
mod support;
use hub_core::{AudioCommand, AudioEvent, Sound, Store, VoiceState};
use std::time::Duration;
use support::Hub;
use voice_client::{ConnectionState, TtsOrigin, TtsStopReason, VoiceEvent};

#[tokio::test(start_paused = true)]
async fn due_timer_waits_for_tts_stop() {
    let start: jiff::Zoned = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    drop(Store::open(dir.path()).unwrap());
    let db = rusqlite::Connection::open(dir.path().join("hub.sqlite3")).unwrap();
    db.execute(
        "INSERT INTO timers (id, label, end_seconds, paused_seconds) VALUES (1, 'tea', ?1, NULL)",
        [start.timestamp().as_second() + 420],
    )
    .unwrap();
    drop(db);
    let mut hub = Hub::start_with_store(start, Store::open(dir.path()).unwrap());
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Idle {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.voice_tx
        .send(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent,
        })
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Speaking {
        hub.ui_rx.changed().await.unwrap();
    }
    tokio::time::advance(Duration::from_secs(420)).await;
    tokio::task::yield_now().await;
    assert!(hub.ui_rx.borrow().alert.is_none());
    hub.voice_tx
        .send(VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed,
        })
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice.state != VoiceState::Listening {
        hub.ui_rx.changed().await.unwrap();
    }
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Play(Sound::TimerChime))
    );
    assert!(hub.ui_rx.borrow().alert.is_some());
}
