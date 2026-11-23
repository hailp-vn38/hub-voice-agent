use jiff::civil::Date;

pub(super) fn weekday_name(date: &Date) -> &'static str {
    match date.weekday().to_monday_one_offset() {
        1 => "Thứ Hai",
        2 => "Thứ Ba",
        3 => "Thứ Tư",
        4 => "Thứ Năm",
        5 => "Thứ Sáu",
        6 => "Thứ Bảy",
        _ => "Chủ nhật",
    }
}
