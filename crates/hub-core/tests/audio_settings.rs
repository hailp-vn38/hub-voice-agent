mod support;
use hub_core::{
    AudioCommand, AudioEvent, ConfigStore, Sensitivity, SettingChange, Sound, Tone, UserInput,
};
use support::Hub;

fn start() -> Hub {
    Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    )
}

#[tokio::test]
async fn volume_and_wake_word_changes_reach_the_audio_worker() {
    let mut hub = start();
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::Volume(0.3)))
        .await
        .unwrap();
    assert_eq!(hub.audio_rx.recv().await, Some(AudioCommand::SetVolume(30)));
    assert_eq!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::SetWakeWord {
            enabled: true,
            sensitivity: Sensitivity::Medium
        })
    );
    hub.input_tx
        .send(UserInput::Setting(SettingChange::Sensitivity(
            Sensitivity::High,
        )))
        .await
        .unwrap();
    hub.audio_rx.recv().await.unwrap();
    assert_eq!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::SetWakeWord {
            enabled: true,
            sensitivity: Sensitivity::High
        })
    );
}

#[tokio::test]
async fn muting_the_mic_disables_wake_word_and_ignores_triggers() {
    let mut hub = start();
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::MicMuted(true)))
        .await
        .unwrap();
    hub.audio_rx.recv().await.unwrap();
    assert_eq!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::SetWakeWord {
            enabled: false,
            sensitivity: Sensitivity::Medium
        })
    );
    hub.audio_tx.send(AudioEvent::WakeWord).await.unwrap();
    hub.input_tx.send(UserInput::OrbTapped).await.unwrap();
    hub.input_tx
        .send(UserInput::PreviewSound("Ping".into()))
        .await
        .unwrap();
    // Không có Ding của lượt hội thoại nào chen vào trước lệnh nghe thử.
    assert_eq!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Play(Sound::Preview(Tone::Ping)))
    );
}

#[tokio::test]
async fn ui_sounds_off_silences_the_ding_but_not_previews() {
    let mut hub = start();
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::UiSounds(false)))
        .await
        .unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.audio_rx.recv().await.unwrap();
    hub.input_tx.send(UserInput::OrbTapped).await.unwrap();
    hub.input_tx
        .send(UserInput::PreviewSound("Reminder".into()))
        .await
        .unwrap();
    assert_eq!(
        hub.audio_rx.recv().await,
        Some(AudioCommand::Play(Sound::Preview(Tone::Reminder)))
    );
}

#[tokio::test]
async fn reading_reminders_is_saved_and_shown() {
    let mut hub = start();
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::ReminderSpeech(false))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice_settings.reminder_speech {
        hub.ui_rx.changed().await.unwrap();
    }
    let saved = ConfigStore::open(hub._dir.path()).unwrap();
    assert!(!saved.get().voice.reminder_speech);
}
