# 04 — Bridge Thời tiết và Lịch

Status: ready-for-human
Blocked by: 01
Spec: [../spec.md](../spec.md) · UI: hub-ui #09, #10

## Việc

- `bridge/weather.rs`: `WeatherData` từ `HubView.weather`. `select-day(key)` → `UserInput::Focus(Some(Focus::Weather(key)))`.
- `bridge/calendar.rs`: `CalendarData` từ `HubView.calendar`. `select-day(day)` → `UserInput::Focus(Some(Focus::Calendar(day)))`.
- `toggle-month`, `open-detail(key)`, `close-detail`: bridge giữ `month_view` và key chi tiết đang mở, set `month-view` và `detail` từ dữ liệu của `HubView.calendar`. Không đổi hợp đồng `CalDetail`. Sự kiện đang mở biến mất → đóng chi tiết.
- Kiểm handler `changed focus-serial` trong `screens/weather.slint`, `components/calendar/{timeline,upcoming}.slint` bỏ qua khi `focus` rỗng.
- Điền bảng Map của hub-ui #09, #10; ghi vào Comments của hub-core #19 rằng chi tiết do bridge giữ nên không cần đổi `CalDetail`.

## Nghiệm thu

- [ ] Tay: màn Thời tiết có dữ liệu Open-Meteo theo vị trí trong Setup; chọn ngày mai → biểu đồ giờ đổi.
- [ ] Tay: màn Lịch có sự kiện Google; chọn ngày → dòng thời gian đổi; xem tháng; mở/đóng chi tiết.
- [ ] Tay (giọng, sau #08): "thứ sáu có lịch gì" → mở Lịch đúng ngày, sự kiện phát sáng.
