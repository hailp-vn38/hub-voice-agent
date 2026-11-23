use super::{day_phase, format_hms};

#[test]
fn day_phase_is_fraction_of_day() {
    assert_eq!(day_phase(0), 0.0);
    assert_eq!(day_phase(12 * 3600), 0.5);
    assert_eq!(day_phase(86_400 + 18 * 3600), 0.75);
}

#[test]
fn formats_hours_minutes_seconds() {
    assert_eq!(format_hms(0), "00:00:00");
    assert_eq!(format_hms(86_400 + 14 * 3600 + 32 * 60 + 5), "14:32:05");
}
