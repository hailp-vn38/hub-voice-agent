use super::format_hms;

#[test]
fn formats_hours_minutes_seconds() {
    assert_eq!(format_hms(0), "00:00:00");
    assert_eq!(format_hms(86_400 + 14 * 3600 + 32 * 60 + 5), "14:32:05");
}
