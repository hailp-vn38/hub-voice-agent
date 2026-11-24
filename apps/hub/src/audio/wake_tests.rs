use super::{WakeDebouncer, threshold};
use std::time::{Duration, Instant};

#[test]
fn wake_word_requires_the_fixed_moderate_threshold() {
    let mut debouncer = WakeDebouncer::default();
    let now = Instant::now();

    assert!(!debouncer.accept(0.49, now));
    assert!(debouncer.accept(0.50, now));
}

#[test]
fn wake_word_is_suppressed_for_two_seconds_after_detection() {
    let mut debouncer = WakeDebouncer::default();
    let now = Instant::now();

    assert!(debouncer.accept(0.8, now));
    assert!(!debouncer.accept(0.8, now + Duration::from_millis(1_999)));
    assert!(debouncer.accept(0.8, now + Duration::from_secs(2)));
}

#[test]
fn sensitivity_moves_the_threshold() {
    use hub_core::Sensitivity;
    let accepts = |sensitivity, score| {
        let mut debouncer = WakeDebouncer {
            threshold: threshold(sensitivity),
            ..WakeDebouncer::default()
        };
        debouncer.accept(score, Instant::now())
    };
    assert!(!accepts(Sensitivity::Low, 0.6));
    assert!(accepts(Sensitivity::High, 0.4));
    assert!(!accepts(Sensitivity::High, 0.2));
    assert!(accepts(Sensitivity::Medium, 0.5));
}
