use super::{Screen, View};

#[test]
fn ambient_starts_after_the_configured_idle_time() {
    let mut view = View::new(0);
    assert!(view.tick(59, Some(60)).is_empty());
    assert!(!view.tick(60, Some(60)).is_empty());
    assert_eq!(view.screen(), Screen::Ambient);
}

#[test]
fn ambient_never_starts_when_disabled() {
    let mut view = View::new(0);
    assert!(view.tick(100_000, None).is_empty());
    assert_eq!(view.screen(), Screen::Center);
}

#[test]
fn screen_all_lists_every_variant_once() {
    // The match has no wildcard: a new variant fails to compile here until it is
    // counted, and then the length check below fails until `ALL` includes it.
    let variants = |screen: Screen| match screen {
        Screen::Center
        | Screen::Menu
        | Screen::Ambient
        | Screen::Weather
        | Screen::Calendar
        | Screen::Home
        | Screen::Timers
        | Screen::Countdowns
        | Screen::Reminders
        | Screen::Photos
        | Screen::Settings => 11,
    };
    assert_eq!(Screen::ALL.len(), variants(Screen::Center));
    for (i, a) in Screen::ALL.iter().enumerate() {
        assert!(!Screen::ALL[i + 1..].contains(a));
    }
}
