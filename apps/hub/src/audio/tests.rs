use super::{capture::Capture, wake};
use hub_core::AudioEvent;

#[test]
fn missing_models_leave_capture_available_without_wake_word_events() {
    let models = tempfile::tempdir().unwrap();
    let mut capture = Capture::new(wake::load_optional(models.path()));

    let events = capture.push(&vec![0; 16_000]);

    assert!(
        events
            .iter()
            .all(|event| !matches!(event, AudioEvent::WakeWord))
    );
}
