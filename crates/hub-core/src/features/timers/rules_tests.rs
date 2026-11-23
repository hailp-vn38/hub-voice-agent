use super::Timers;

#[test]
fn pause_resume_preserves_remaining_time_and_due_uses_absolute_time() {
    let mut timers = Timers::default();
    timers.start("egg".into(), 60, 100).unwrap();
    timers.adjust("egg", None, Some(true), 120).unwrap();
    assert!(timers.due(500).is_empty());
    timers.adjust("egg", None, Some(false), 500).unwrap();
    assert!(timers.due(539).is_empty());
    assert_eq!(timers.due(540).len(), 1);
}
