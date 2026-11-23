//! Dữ liệu lịch giả + các phép dựng model từ dữ liệu đó (thuần, không đụng Slint App).
//! Demo giả định hôm nay là thứ Hai 23/11/2026; ngày = số ngày lệch so với hôm nay.
use super::columns::assign;
use crate::{CalBlock, CalDay, CalKind, CalMonthDay, CalUpcoming, PendingState};

#[derive(Clone)]
pub struct Ev {
    pub key: String,
    pub day: i32,
    pub kind: CalKind,
    pub title: String,
    pub start: i32,
    pub end: i32,
    pub all_day: bool,
    pub location: String,
    pub description: String,
    pub fresh: bool,
    pub pending: PendingState,
}

pub fn ev(key: &str, day: i32, kind: CalKind, title: &str, start: i32, end: i32) -> Ev {
    Ev {
        key: key.into(), day, kind, title: title.into(), start, end, all_day: false,
        location: String::new(), description: String::new(), fresh: false, pending: PendingState::None,
    }
}

pub fn all_day(key: &str, day: i32, title: &str) -> Ev {
    Ev { all_day: true, ..ev(key, day, CalKind::Event, title, 0, 1440) }
}

pub fn hm(m: i32) -> String {
    format!("{:02}:{:02}", m / 60, m % 60)
}

pub fn day_key(day: i32) -> String {
    format!("d{day}")
}

pub fn parse_day(key: &str) -> i32 {
    key.trim_start_matches('d').parse().unwrap_or(0)
}

pub fn week() -> Vec<(String, String)> {
    ["T2", "T3", "T4", "T5", "T6", "T7", "CN"].iter().enumerate()
        .map(|(i, w)| ((*w).to_string(), (23 + i).to_string())).collect()
}

pub fn week_days(events: &[Ev]) -> Vec<CalDay> {
    week().into_iter().enumerate().map(|(i, (w, d))| CalDay {
        key: day_key(i as i32).into(), weekday: w.into(), day: d.into(),
        dots: events.iter().filter(|e| e.day == i as i32 && e.pending != PendingState::Failed).count() as i32,
        today: i == 0,
    }).collect()
}

pub fn blocks(events: &[Ev], sel: i32, now_min: i32) -> Vec<CalBlock> {
    let mut day: Vec<&Ev> = events.iter().filter(|e| e.day == sel).collect();
    day.sort_by_key(|e| (e.start, e.end));
    let timed: Vec<usize> = (0..day.len()).filter(|&i| !day[i].all_day).collect();
    let cols = assign(&timed.iter().map(|&i| (day[i].start, day[i].end.max(day[i].start + 30))).collect::<Vec<_>>());
    day.iter().enumerate().map(|(i, e)| {
        let (col, n) = timed.iter().position(|&t| t == i).map_or((0, 1), |p| cols[p]);
        CalBlock {
            key: e.key.as_str().into(), kind: e.kind, title: e.title.as_str().into(),
            time: if e.all_day { "".into() } else { hm(e.start).into() },
            start_min: e.start, end_min: e.end, all_day: e.all_day,
            ongoing: sel == 0 && !e.all_day && e.start <= now_min && now_min < e.end,
            fresh: e.fresh, pending: e.pending, col, cols: n,
        }
    }).collect()
}

/// 5 ngày kế tiếp sau `sel` có sự kiện; mỗi ngày một dòng, ngày nhiều sự kiện gộp "+N".
pub fn upcoming(events: &[Ev], sel: i32) -> Vec<CalUpcoming> {
    let mut days: Vec<i32> = events.iter().filter(|e| e.day > sel && e.pending != PendingState::Failed).map(|e| e.day).collect();
    days.sort();
    days.dedup();
    days.into_iter().take(5).map(|d| {
        let mut of_day: Vec<&Ev> = events.iter().filter(|e| e.day == d).collect();
        of_day.sort_by_key(|e| e.start);
        let first = of_day[0];
        CalUpcoming {
            key: first.key.as_str().into(),
            date: if 23 + d > 30 { format!("{}/12", 23 + d - 30) } else { format!("{}/11", 23 + d) }.into(),
            time: if first.all_day { "".into() } else { hm(first.start).into() },
            title: first.title.as_str().into(),
            more: of_day.len() as i32 - 1,
            fresh: first.fresh,
        }
    }).collect()
}

/// Lưới 6x7 (thứ Hai đầu tuần). `next`: false = tháng 11, true = tháng 12/2026.
pub fn month(events: &[Ev], next: bool) -> Vec<CalMonthDay> {
    let (lead, len, off) = if next { (1, 31, 7) } else { (6, 30, -23) }; // off: offset của ngày d là d + off
    (0..42).map(|i| {
        let d = i - lead + 1; // ngày trong tháng (có thể <1 hoặc >len)
        let day_off = d + off;
        let in_month = (1..=len).contains(&d);
        let shown = if d < 1 { (if next { 30 } else { 31 }) + d } else if d > len { d - len } else { d };
        CalMonthDay {
            key: day_key(day_off).into(), day: shown.to_string().into(),
            dots: events.iter().filter(|e| e.day == day_off).count() as i32,
            in_month, today: day_off == 0,
        }
    }).collect()
}

pub fn seed() -> Vec<Ev> {
    let e = CalKind::Event;
    let mut v = vec![
        all_day("a0", 0, "Sinh nhật Lan"),
        ev("e1", 0, e, "Họp nhóm", 9 * 60, 10 * 60),
        ev("e2", 0, e, "Gọi khách hàng", 9 * 60 + 30, 11 * 60),
        ev("e3", 0, e, "Ăn trưa với Nam", 12 * 60 + 30, 13 * 60 + 30),
        ev("r1", 0, CalKind::Reminder, "Uống thuốc", 15 * 60, 15 * 60 + 30),
        ev("e4", 0, e, "Đá bóng", 18 * 60, 19 * 60 + 30),
        ev("e5", 1, e, "Họp phụ huynh", 10 * 60, 11 * 60),
        all_day("a2", 2, "Ngày nghỉ"),
        ev("e6", 2, e, "Dọn nhà", 14 * 60, 15 * 60),
        ev("e7", 4, e, "Đưa con đi học", 8 * 60, 9 * 60),
        ev("e8", 4, e, "Họp dự án", 8 * 60 + 30, 10 * 60),
        ev("r2", 4, CalKind::Reminder, "Gọi bảo hiểm", 12 * 60, 12 * 60 + 30),
        ev("e9", 4, e, "Xem phim", 19 * 60, 21 * 60),
        ev("e10", 5, e, "Chạy bộ", 7 * 60, 8 * 60),
        ev("e11", 6, e, "Cơm nhà bà", 11 * 60, 13 * 60),
        ev("e12", 12, e, "Họp quý", 9 * 60, 11 * 60),
        ev("e13", 20, e, "Sinh nhật Minh", 15 * 60, 17 * 60),
    ];
    v[1].location = "Phòng họp tầng 3".into();
    v[1].description = "Rà soát tiến độ tuần.".into();
    v
}

#[cfg(test)]
#[path = "data_tests.rs"]
mod tests;
