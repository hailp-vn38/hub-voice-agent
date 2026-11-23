use super::{blocks, seed, upcoming};

#[test]
fn upcoming_groups_by_day_and_skips_selected_day() {
    let u = upcoming(&seed(), 3);
    // ngày 4 có 4 sự kiện -> một dòng "+3"; ngày 3 (đã chọn) không xuất hiện.
    assert_eq!(u[0].date.as_str(), "27/11");
    assert_eq!(u[0].more, 3);
    assert_eq!(u.len(), 5);
}

#[test]
fn overlapping_events_get_separate_columns() {
    let b = blocks(&seed(), 0, 0);
    let by = |k: &str| b.iter().find(|x| x.key == k).unwrap().clone();
    assert_eq!((by("e1").col, by("e1").cols), (0, 2));
    assert_eq!((by("e2").col, by("e2").cols), (1, 2));
    assert_eq!(by("e3").cols, 1);
}

#[test]
fn ongoing_only_for_today_and_within_range() {
    let b = blocks(&seed(), 0, 9 * 60 + 45);
    assert!(b.iter().find(|x| x.key == "e1").unwrap().ongoing);
    assert!(!b.iter().find(|x| x.key == "e3").unwrap().ongoing);
}
