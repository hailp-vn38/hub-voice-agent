use super::{Reminder, ReminderRepeat, parse_at};
use jiff::{Zoned, tz::TimeZone};

fn zoned(value: &str) -> Zoned {
    value.parse().unwrap()
}

#[test]
fn reminder_time_accepts_zoned_rfc3339_and_local_forms() {
    let tz = TimeZone::get("Asia/Ho_Chi_Minh").unwrap();
    let expected = zoned("2026-11-23T09:00:00+07:00[Asia/Ho_Chi_Minh]");
    for input in [
        "2026-11-23T09:00:00+07:00[Asia/Ho_Chi_Minh]",
        "2026-11-23T09:00:00+07:00",
        "2026-11-23T02:00:00Z",
        "2026-11-23T09:00:00",
    ] {
        assert_eq!(parse_at(input, &tz), Some(expected.clone()), "{input}");
    }
    assert_eq!(parse_at("tomorrow", &tz), None);
}

#[test]
fn repeating_reminder_catches_up_past_missed_days_in_one_step() {
    let reminder = Reminder {
        id: 1,
        label: "Thuốc".into(),
        at: "2026-11-20T09:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        repeat: ReminderRepeat::Daily,
        weekdays: vec![],
        scheduled_at: None,
    };
    let now = zoned("2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]");
    assert_eq!(
        reminder.next_current(&now).unwrap(),
        "2026-11-24T09:00:00+07:00[Asia/Ho_Chi_Minh]"
    );
    let now = zoned("2026-11-23T09:30:00+07:00[Asia/Ho_Chi_Minh]");
    assert_eq!(
        reminder.next_current(&now).unwrap(),
        "2026-11-23T09:00:00+07:00[Asia/Ho_Chi_Minh]"
    );
}
