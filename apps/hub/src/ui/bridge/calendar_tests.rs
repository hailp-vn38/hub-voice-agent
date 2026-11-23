use super::CalendarInteraction;
use hub_core::present::{CalendarDetailView, CalendarUpcomingView};

fn detail(key: &str) -> CalendarDetailView {
    CalendarDetailView {
        key: key.into(),
        title: format!("Sự kiện {key}"),
        time: "09:00".into(),
        location: String::new(),
        description: String::new(),
    }
}

fn upcoming(key: &str) -> CalendarUpcomingView {
    CalendarUpcomingView {
        key: key.into(),
        date: "28/11".into(),
        time: "09:00".into(),
        title: format!("Sự kiện {key}"),
        more: 0,
        fresh: false,
    }
}

#[test]
fn month_view_toggles_and_day_selection_returns_to_timeline() {
    let mut interaction = CalendarInteraction::default();

    interaction.toggle_month();
    assert!(interaction.month_view);

    interaction.select_day();
    assert!(!interaction.month_view);
}

#[test]
fn detail_is_selected_by_key_and_closes_when_the_event_disappears() {
    let mut interaction = CalendarInteraction::default();
    let details = [detail("event-1"), detail("event-2")];

    interaction.open_detail("event-2".into());
    assert_eq!(interaction.detail(&details, &[]).unwrap().key, "event-2");

    assert!(interaction.detail(&[detail("event-1")], &[]).is_none());
    assert!(interaction.detail_key.is_none());
}

#[test]
fn upcoming_detail_uses_its_summary_until_the_event_is_selected() {
    let mut interaction = CalendarInteraction::default();
    interaction.open_detail("event-1".into());

    let result = interaction.detail(&[], &[upcoming("event-1")]).unwrap();

    assert_eq!(result.title, "Sự kiện event-1");
    assert_eq!(result.time, "28/11 · 09:00");
    assert!(result.location.is_empty());
    assert!(result.description.is_empty());
}

#[test]
fn closing_detail_clears_the_selected_key() {
    let mut interaction = CalendarInteraction::default();
    interaction.open_detail("event-1".into());

    interaction.close_detail();

    assert!(interaction.detail(&[detail("event-1")], &[]).is_none());
}
