use hub_core::{
    CalendarEvent, Focus, Reminder, ReminderRepeat, UiState, present, present::CalendarKind,
};

fn event(id: &str, start: &str, end: &str) -> CalendarEvent {
    CalendarEvent {
        location: None,
        description: None,
        id: id.into(),
        summary: id.into(),
        start: start.into(),
        end: end.into(),
    }
}

fn timed_event(id: &str, start_min: i32, end_min: i32) -> CalendarEvent {
    let stamp = |minute: i32| format!("2026-11-23T{:02}:{:02}:00+07:00", minute / 60, minute % 60);
    event(id, &stamp(start_min), &stamp(end_min))
}

#[test]
fn overlapping_events_share_columns_and_all_day_events_stay_out_of_timed_columns() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.calendar = vec![
        event("all-day", "2026-11-23", "2026-11-24"),
        event(
            "first",
            "2026-11-23T10:00:00+07:00",
            "2026-11-23T12:00:00+07:00",
        ),
        event(
            "second",
            "2026-11-23T10:30:00+07:00",
            "2026-11-23T11:30:00+07:00",
        ),
        event(
            "third",
            "2026-11-23T11:00:00+07:00",
            "2026-11-23T13:00:00+07:00",
        ),
    ];

    let calendar = present(&state).calendar;
    let all_day = calendar
        .blocks
        .iter()
        .find(|block| block.key == "all-day")
        .unwrap();
    assert!(all_day.all_day);
    assert_eq!((all_day.col, all_day.cols), (0, 1));

    let timed: Vec<_> = calendar
        .blocks
        .iter()
        .filter(|block| !block.all_day)
        .map(|block| (block.key.as_str(), block.col, block.cols))
        .collect();
    assert_eq!(
        timed,
        [("first", 0, 3), ("second", 1, 3), ("third", 2, 3)],
        "blocks: {:?}",
        calendar.blocks
    );
}

#[test]
fn disjoint_calendar_events_get_the_full_timeline_width() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.calendar = vec![timed_event("first", 60, 120), timed_event("next", 120, 180)];

    let columns: Vec<_> = present(&state)
        .calendar
        .blocks
        .into_iter()
        .map(|block| (block.col, block.cols))
        .collect();
    assert_eq!(columns, [(0, 1), (0, 1)]);
}

#[test]
fn calendar_reuses_a_column_after_a_gap_inside_an_overlap_cluster() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.calendar = vec![
        timed_event("wide", 60, 180),
        timed_event("short", 90, 120),
        timed_event("reuse", 130, 170),
    ];

    let columns: Vec<_> = present(&state)
        .calendar
        .blocks
        .into_iter()
        .map(|block| (block.col, block.cols))
        .collect();
    assert_eq!(columns, [(0, 2), (1, 2), (1, 2)]);
}

#[test]
fn separate_calendar_overlap_clusters_are_independent() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.calendar = vec![
        timed_event("first", 0, 60),
        timed_event("second", 30, 90),
        timed_event("later", 300, 360),
    ];

    let columns: Vec<_> = present(&state)
        .calendar
        .blocks
        .into_iter()
        .map(|block| (block.col, block.cols))
        .collect();
    assert_eq!(columns, [(0, 2), (1, 2), (0, 1)]);
}

#[test]
fn calendar_focus_on_an_event_selects_its_start_day() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.view.focus = Some(Focus::Calendar("friday-event".into()));
    state.calendar = vec![
        event(
            "friday-event",
            "2026-11-27T09:00:00+07:00",
            "2026-11-27T10:00:00+07:00",
        ),
        event(
            "monday-event",
            "2026-11-23T09:00:00+07:00",
            "2026-11-23T10:00:00+07:00",
        ),
    ];

    let calendar = present(&state).calendar;
    assert_eq!(calendar.selected_day, "2026-11-27");
    assert_eq!(
        calendar
            .blocks
            .iter()
            .map(|block| block.key.as_str())
            .collect::<Vec<_>>(),
        ["friday-event"]
    );
}

#[test]
fn calendar_blocks_include_a_reminder_on_the_selected_day() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.reminders = vec![Reminder {
        id: 7,
        label: "Uống thuốc".into(),
        at: "2026-11-23T10:30:00+07:00[Asia/Ho_Chi_Minh]".into(),
        repeat: ReminderRepeat::None,
        weekdays: Vec::new(),
        scheduled_at: None,
    }];

    let calendar = present(&state).calendar;
    let reminder = calendar
        .blocks
        .iter()
        .find(|block| block.kind == CalendarKind::Reminder)
        .unwrap();
    assert_eq!(reminder.key, "reminder-7");
    assert_eq!(reminder.title, "Uống thuốc");
    assert_eq!((reminder.start_min, reminder.end_min), (630, 660));
}

#[test]
fn calendar_present_builds_week_month_upcoming_details_and_sync_status() {
    let mut state = UiState::default();
    state.now = "2026-11-23T10:15:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.view.focus = Some(Focus::Calendar("friday-first".into()));
    state.events.focus = 12;
    state.calendar_online = Some(true);
    state.calendar_last_sync = Some(
        "2026-11-23T10:14:00+07:00"
            .parse::<jiff::Timestamp>()
            .unwrap()
            .as_second(),
    );
    let mut friday = event(
        "friday-first",
        "2026-11-27T09:00:00+07:00",
        "2026-11-27T10:00:00+07:00",
    );
    friday.location = Some("Phòng 3".into());
    friday.description = Some("Rà soát tiến độ tuần.".into());
    state.calendar = vec![
        event(
            "today",
            "2026-11-23T09:00:00+07:00",
            "2026-11-23T10:00:00+07:00",
        ),
        friday,
        event(
            "friday-second",
            "2026-11-27T09:30:00+07:00",
            "2026-11-27T10:30:00+07:00",
        ),
        event(
            "saturday-first",
            "2026-11-28T08:00:00+07:00",
            "2026-11-28T09:00:00+07:00",
        ),
        event(
            "saturday-second",
            "2026-11-28T10:00:00+07:00",
            "2026-11-28T11:00:00+07:00",
        ),
        event(
            "sunday",
            "2026-11-29T12:00:00+07:00",
            "2026-11-29T13:00:00+07:00",
        ),
    ];

    let calendar = present(&state).calendar;
    assert_eq!(calendar.month_title, "Tháng 11");
    assert_eq!(calendar.synced, "10:14");
    assert!(!calendar.stale);
    state.calendar_online = Some(false);
    assert!(present(&state).calendar.stale);
    assert_eq!(calendar.now_min, 615);
    assert_eq!(calendar.focus, "friday-first");
    assert_eq!(calendar.focus_serial, 12);
    assert_eq!(calendar.week.len(), 7);
    assert_eq!(calendar.week[0].key, "2026-11-23");
    assert_eq!(calendar.week[0].weekday, "T2");
    let friday_day = calendar
        .week
        .iter()
        .find(|day| day.key == "2026-11-27")
        .unwrap();
    assert_eq!(friday_day.dots, 2);
    assert!(!friday_day.today);
    assert_eq!(calendar.month.len(), 42);
    assert_eq!(calendar.month.iter().filter(|day| day.in_month).count(), 30);
    assert_eq!(calendar.upcoming.len(), 2);
    assert_eq!(calendar.upcoming[0].key, "saturday-first");
    assert_eq!(calendar.upcoming[0].date, "28/11");
    assert_eq!(calendar.upcoming[0].time, "08:00");
    assert_eq!(calendar.upcoming[0].more, 1);
    assert_eq!(calendar.details.len(), 2);
    assert_eq!(calendar.details[0].key, "friday-first");
    assert_eq!(calendar.details[0].title, "friday-first");
    assert_eq!(calendar.details[0].time, "09:00 – 10:00");
    assert_eq!(calendar.details[0].location, "Phòng 3");
    assert_eq!(calendar.details[0].description, "Rà soát tiến độ tuần.");
}
