use hub_core::{Reminder, ReminderRepeat};

#[test]
fn daily_reminder_crosses_month_boundary() {
    let reminder = Reminder {
        id: 1,
        label: "Thuốc".into(),
        at: "2026-01-31T09:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        repeat: ReminderRepeat::Daily,
        weekdays: vec![],
    };
    assert_eq!(
        reminder.next_occurrence().unwrap(),
        "2026-02-01T09:00:00+07:00[Asia/Ho_Chi_Minh]"
    );
}

#[test]
fn weekly_reminder_crosses_sunday_to_monday() {
    let reminder = Reminder {
        id: 1,
        label: "Thuốc".into(),
        at: "2026-11-22T09:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        repeat: ReminderRepeat::Weekly,
        weekdays: vec![1],
    };
    assert_eq!(
        reminder.next_occurrence().unwrap(),
        "2026-11-23T09:00:00+07:00[Asia/Ho_Chi_Minh]"
    );
}

#[test]
fn overdue_reminder_only_alerts_within_one_hour() {
    let reminder = Reminder {
        id: 1,
        label: "Thuốc".into(),
        at: "2026-11-22T09:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        repeat: ReminderRepeat::None,
        weekdays: vec![],
    };
    assert_eq!(
        reminder.due(
            "2026-11-22T09:30:00+07:00[Asia/Ho_Chi_Minh]"
                .parse()
                .unwrap()
        ),
        Some(hub_core::ReminderDue::Alert)
    );
    assert_eq!(
        reminder.due(
            "2026-11-22T11:00:00+07:00[Asia/Ho_Chi_Minh]"
                .parse()
                .unwrap()
        ),
        Some(hub_core::ReminderDue::Missed)
    );
}

#[test]
fn reminder_tools_are_advertised() {
    let names: Vec<_> = hub_core::tool_definitions()
        .into_iter()
        .map(|tool| tool.name)
        .collect();
    for name in [
        "hub_reminder_create",
        "hub_reminder_list",
        "hub_reminder_cancel",
        "hub_reminder_snooze",
    ] {
        assert!(names.iter().any(|item| item == name), "missing {name}");
    }
}
