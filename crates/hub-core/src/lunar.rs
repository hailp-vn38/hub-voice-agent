//! Vietnamese lunar calendar, using the astronomical algorithm by Hồ Ngọc Đức (UTC+7).
use jiff::civil::{Date, date};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LunarDate {
    pub year: i16,
    pub month: i8,
    pub day: i8,
    pub leap: bool,
}

pub(crate) fn julian_day(d: Date) -> i32 {
    let (day, month, year) = (d.day() as i32, d.month() as i32, d.year() as i32);
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}
fn new_moon(k: i32) -> i32 {
    let t = k as f64 / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let r = PI / 180.0;
    let mut n = 2415020.75933 + 29.53058868 * k as f64 + 0.0001178 * t2 - 0.000000155 * t3;
    n += 0.00033 * ((166.56 + 132.87 * t - 0.009173 * t2) * r).sin();
    let m = (359.2242 + 29.10535608 * k as f64 - 0.0000333 * t2 - 0.00000347 * t3) * r;
    let mp = (306.0253 + 385.81691806 * k as f64 + 0.0107306 * t2 + 0.00001236 * t3) * r;
    let f = (21.2964 + 390.67050646 * k as f64 - 0.0016528 * t2 - 0.00000239 * t3) * r;
    let c = (0.1734 - 0.000393 * t) * m.sin() + 0.0021 * (2.0 * m).sin() - 0.4068 * mp.sin()
        + 0.0161 * (2.0 * mp).sin()
        - 0.0004 * (3.0 * mp).sin()
        + 0.0104 * (2.0 * f).sin()
        - 0.0051 * (m + mp).sin()
        - 0.0074 * (m - mp).sin()
        + 0.0004 * (2.0 * f + m).sin()
        - 0.0004 * (2.0 * f - m).sin()
        - 0.0006 * (2.0 * f + mp).sin()
        + 0.0010 * (2.0 * f - mp).sin()
        + 0.0005 * (2.0 * mp + m).sin();
    let delta = if t < -11.0 {
        0.001 + 0.000839 * t + 0.0002261 * t2 - 0.00000845 * t3 - 0.000000081 * t * t3
    } else {
        -0.000278 + 0.000265 * t + 0.000262 * t2
    };
    (n + c - delta + 0.5 + 7.0 / 24.0).floor() as i32
}
fn sun(day: i32) -> i32 {
    let t = (day as f64 - 0.5 - 7.0 / 24.0 - 2451545.0) / 36525.0;
    let r = PI / 180.0;
    let m = (357.52910 + 35999.05030 * t - 0.0001559 * t * t - 0.00000048 * t * t * t) * r;
    let l = 280.46645
        + 36000.76983 * t
        + 0.0003032 * t * t
        + (1.914600 - 0.004817 * t - 0.000014 * t * t) * m.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
        + 0.000290 * (3.0 * m).sin();
    ((l * r).rem_euclid(2.0 * PI) / PI * 6.0).floor() as i32
}
fn month11(year: i16) -> i32 {
    let k = ((julian_day(date(year, 12, 31)) - 2415021) as f64 / 29.530588853).floor() as i32;
    let n = new_moon(k);
    if sun(n) >= 9 { new_moon(k - 1) } else { n }
}
fn leap_offset(a: i32) -> i32 {
    let k = ((a as f64 - 2415021.076998695) / 29.530588853 + 0.5).floor() as i32;
    let mut last = 0;
    let mut i = 1;
    let mut arc = sun(new_moon(k + i));
    while arc != last && i < 14 {
        last = arc;
        i += 1;
        arc = sun(new_moon(k + i));
    }
    i - 1
}
pub fn to_lunar(solar: Date) -> LunarDate {
    let day = julian_day(solar);
    let k = ((day - 2415021) as f64 / 29.530588853).floor() as i32;
    let mut start = new_moon(k + 1);
    if start > day {
        start = new_moon(k);
    }
    let mut a = month11(solar.year());
    let mut b = a;
    let mut year = solar.year();
    if a >= start {
        a = month11(year - 1);
    } else {
        year += 1;
        b = month11(year);
    }
    let diff = (start - a) / 29;
    let mut month = diff + 11;
    let mut leap = false;
    if b - a > 365 {
        let offset = leap_offset(a);
        if diff >= offset {
            month = diff + 10;
            leap = diff == offset;
        }
    }
    if month > 12 {
        month -= 12;
    }
    if month >= 11 && diff < 4 {
        year -= 1;
    }
    LunarDate {
        year,
        month: month as i8,
        day: (day - start + 1) as i8,
        leap,
    }
}
pub fn to_solar(lunar: LunarDate) -> Option<Date> {
    if !(1..=12).contains(&lunar.month) || !(1..=30).contains(&lunar.day) {
        return None;
    }
    // Each lunar month begins at a new moon; inspect only month starts in the civil year.
    let start = date(lunar.year, 1, 1);
    let start_jd = julian_day(start);
    let k = ((start_jd - 2415021) as f64 / 29.530588853).floor() as i32;
    for month_offset in 0..16 {
        let start_offset = new_moon(k + month_offset) - start_jd;
        let solar = start
            .checked_add(jiff::Span::new().days((start_offset + i32::from(lunar.day) - 1) as i64))
            .ok()?;
        if to_lunar(solar) == lunar {
            return Some(solar);
        }
    }
    None
}
