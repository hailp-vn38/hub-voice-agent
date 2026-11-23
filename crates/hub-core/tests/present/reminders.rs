use hub_core::{Reminder, ReminderHistory, ReminderRepeat, UiState, present};

#[test]
fn present_groups_reminders_and_formats_until_and_recent_history() {
    let mut state = UiState::default();
    state.now = "2025-11-24T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.reminders = vec![
        Reminder {
            id: 1,
            label: "Uống thuốc".into(),
            at: "2025-11-24T12:25:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::Daily,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
        Reminder {
            id: 2,
            label: "Khám răng".into(),
            at: "2025-11-25T09:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::None,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
        Reminder {
            id: 3,
            label: "Gọi mẹ".into(),
            at: "2025-11-28T18:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::None,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
    ];
    let past = state.now.checked_sub(jiff::Span::new().minutes(5)).unwrap();
    state.reminder_history.push(ReminderHistory {
        reminder: state.reminders[1].clone(),
        missed: true,
        at_seconds: past.timestamp().as_second(),
    });

    let reminders = present(&state).reminders;
    assert_eq!(reminders.title, "3 sắp tới");
    assert!(reminders.has_next);
    assert_eq!(reminders.next.key, "1");
    assert_eq!(reminders.next.time, "12:25");
    assert_eq!(reminders.next.until, "còn 25 phút");
    assert_eq!(reminders.groups[0].label, "Hôm nay");
    assert_eq!(reminders.groups[0].items[0].repeat_text, "Hằng ngày");
    assert!(reminders.groups[0].items[0].repeats);
    assert_eq!(reminders.groups[1].label, "Ngày mai");
    assert_eq!(reminders.groups[1].items[0].repeat_text, "");
    assert!(!reminders.groups[1].items[0].repeats);
    assert_eq!(reminders.groups[2].label, "Thứ Sáu 28/11");
    assert_eq!(reminders.recent[0].time, "11:55");
    assert!(reminders.recent[0].missed);

    state.now = "2025-11-24T12:05:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    assert_eq!(present(&state).reminders.next.until, "còn 20 phút");
}
