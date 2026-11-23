use tokio::sync::{mpsc, watch};
use voice_client::VoiceCommand;

use super::{Effect, execute};
use crate::UiState;
use crate::runtime::jobs::Jobs;

#[tokio::test]
async fn voice_effect_sends_command_to_voice_channel() {
    let (voice_tx, mut voice_rx) = mpsc::channel(1);
    let (audio_tx, _audio_rx) = mpsc::channel(1);
    let (ui_tx, _ui_rx) = watch::channel(UiState::default());

    let mut jobs = Jobs::new();
    execute(
        Effect::Voice(VoiceCommand::Connect),
        &ui_tx,
        &voice_tx,
        &audio_tx,
        &mut jobs,
    )
    .await;

    assert!(matches!(voice_rx.recv().await, Some(VoiceCommand::Connect)));
}
