use super::{Inputs, to_core_screen, to_ui_screen};
use hub_core::UserInput;
use std::time::{Duration, Instant};

#[test]
fn every_hub_core_screen_round_trips_through_slint() {
    let screens = [
        hub_core::Screen::Center,
        hub_core::Screen::Menu,
        hub_core::Screen::Ambient,
        hub_core::Screen::Weather,
        hub_core::Screen::Calendar,
        hub_core::Screen::Home,
        hub_core::Screen::Timers,
        hub_core::Screen::Countdowns,
        hub_core::Screen::Reminders,
        hub_core::Screen::Photos,
        hub_core::Screen::Settings,
    ];

    for core_screen in screens {
        let slint_screen = to_ui_screen(core_screen);
        assert_eq!(to_core_screen(slint_screen), Some(core_screen));
    }
    assert_eq!(to_core_screen(crate::Screen::FirstBoot), None);
}

#[test]
fn activity_is_sent_once_during_a_one_second_burst() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let mut inputs = Inputs::new(tx);
    let start = Instant::now();

    for tenth in 0..10 {
        inputs.activity_at(start + Duration::from_millis(tenth * 100));
    }

    assert!(matches!(rx.try_recv(), Ok(UserInput::Activity)));
    assert!(rx.try_recv().is_err());

    inputs.activity_at(start + Duration::from_secs(1));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Activity)));
}
