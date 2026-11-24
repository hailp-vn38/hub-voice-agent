use super::diff;
use hub_core::{Screen, UiState};

#[test]
fn first_view_writes_every_global() {
    let view = hub_core::present(&UiState::default());

    assert_eq!(diff(None, &view, None).len(), 16);
}

#[test]
fn unchanged_view_writes_nothing() {
    let view = hub_core::present(&UiState::default());

    assert!(diff(Some(&view), &view, None).is_empty());
}

#[test]
fn only_changed_parts_are_written() {
    let shown = hub_core::present(&UiState::default());
    let mut view = shown.clone();
    view.nav = if shown.nav == Screen::Weather {
        Screen::Calendar
    } else {
        Screen::Weather
    };

    assert_eq!(diff(Some(&shown), &view, None).len(), 1);
}
