use super::*;

#[test]
fn format_remaining_minutes_seconds() {
    assert_eq!(format_remaining(402), "6:42");
    assert_eq!(format_remaining(59), "0:59");
    assert_eq!(format_remaining(3600), "1:00:00");
}

#[test]
fn counts_down_enters_final_seconds_then_finishes() {
    let mut t = Sim::new("a", "Trứng", 12);
    t.tick();
    t.tick();
    assert_eq!(t.remaining, 10);
    assert_eq!(t.phase(), Phase::FinalSeconds);
    for _ in 0..10 {
        t.tick();
    }
    assert_eq!(t.phase(), Phase::Finished);
    assert_eq!(t.remaining, 0);
}

#[test]
fn paused_does_not_count() {
    let mut t = Sim::new("a", "x", 100);
    t.paused = true;
    t.tick();
    assert_eq!(t.remaining, 100);
    assert_eq!(t.phase(), Phase::Paused);
}

#[test]
fn add_minute_extends_total_and_progress() {
    let mut t = Sim::new("a", "x", 60);
    t.add_minute();
    assert_eq!((t.remaining, t.total), (120, 120));
    assert!((t.progress() - 1.0).abs() < 1e-6);
}

#[test]
fn detail_text() {
    let mut t = Sim::new("a", "x", 420);
    assert_eq!(t.detail(14 * 3600 + 25 * 60), "7 phút · xong lúc 14:32");
    t.remaining = 0;
    t.mark_finished(14 * 3600 + 32 * 60);
    assert_eq!(t.detail(15 * 3600), "Đã hết giờ lúc 14:32");
}
