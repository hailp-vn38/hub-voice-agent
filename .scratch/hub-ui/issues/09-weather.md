# 09 — Màn Thời tiết

Status: ready-for-human
Blocked by: 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Thời tiết"

## Việc

- `screens/weather.slint` + component con trong `components/weather/` (đường 24 giờ, hàng 7 ngày, nền động) để giữ file < 300 dòng.
- **Đầu màn:** "‹ Thời tiết · {vị trí}" + nhãn **"tạm xem"** khi đang xem nơi khác; "Cập nhật HH:MM" bên phải, badge offline nếu cũ.
- **Cột trái — Hiện tại:** icon, nhiệt độ lớn, mô tả, cảm giác như, độ ẩm, gió, UV, AQI + mức tô màu theo mức, giờ mọc/lặn.
- **Cột giữa — 24 giờ:** đường nhiệt độ mượt, nhãn mỗi 6 giờ, đánh dấu điểm cao nhất; cột mờ khả năng mưa theo giờ.
- **Cột phải — 7 ngày:** thứ, icon, thanh khoảng nhiệt trên cùng một thang cả tuần; hàng "Mai" nổi bật.
- **Nền động** theo `background`: quang (vầng sáng xoay chậm / sao nhấp nháy ban đêm), nhiều mây (2–3 đám trôi lệch tốc), mưa (mây + 20–40 hạt xiên), giông (mưa + chớp thưa), sương mù (lớp mờ trôi). Màu nền ngả theo `is-day`. Giới hạn số hạt, chỉ transform/opacity.
- **Chạm:** điểm trên đường 24 giờ → nhiệt độ, mưa, gió của giờ đó; chạm ngày → đường giữa chuyển sang ngày đó (`select-day(key)`). Không cuộn ngang.
- **Focus giọng nói:** `focus-serial` đổi → làm sáng phần `focus` (ngày, giờ, dòng AQI); đường giữa chuyển theo ngày được focus.
- `contract/weather.slint`:
  - `export enum WeatherBg { clear, cloudy, rain, storm, fog }`.
  - `WeatherNow { icon, temp, summary, feels, humidity, wind, uv, aqi, aqi-level, aqi-color: color, sunrise, sunset }`, `WeatherHour { key, label, temp: float, rain: float, wind, icon }`, `WeatherDay { key, label, icon, low: float, high: float, low-text, high-text, is-tomorrow: bool }`.
  - `global WeatherData { in property <string> location; in property <bool> preview; in property <string> updated; in property <bool> stale; in property <WeatherBg> background; in property <bool> is-day; in property <WeatherNow> now; in property <[WeatherHour]> hours; in property <string> selected-day; in property <[WeatherDay]> days; in property <float> week-min; in property <float> week-max; in property <string> focus; in property <int> focus-serial; callback select-day(string); }`. `hours` là 24 giờ của `selected-day`.

## Demo

Fixture: Hà Nội (nhà) và Đà Nẵng (tạm xem). Kịch bản: mỗi kiểu nền (5), ngày/đêm, "Mai có mưa không?" (focus ngày mai), "Chiều nay mấy độ?" (focus giờ), "Không khí thế nào?" (focus AQI), "Thời tiết Đà Nẵng" (tạm xem), "Offline".

## Nghiệm thu

- [ ] Ba cột đúng nội dung như concept, không tràn ở `text-scale` lớn.
- [ ] Thanh nhiệt 7 ngày dùng chung thang `week-min..week-max`.
- [ ] Chạm ngày đổi đường 24 giờ; chạm điểm hiện chi tiết giờ đó.
- [ ] "tạm xem" hiện rõ khi `preview`.
- [ ] Đo fps trên Pi cho từng nền động (ready-for-human); nền nào < 30 fps thì giảm hạt và ghi lại.

## Map với hub-core

Map: done (hub-mac #04)

| Hợp đồng | UiState / UserInput |
|---|---|
| `WeatherData.location` / `.preview` | `HubView.weather.location` / `.preview` |
| `.updated` / `.stale` | `HubView.weather.updated` / `.stale` |
| `.background` / `.is-day` | `HubView.weather.background` / `.is_day` |
| `.now` | `HubView.weather.now`; `aqi_color` string đổi sang Slint `Color` |
| `.hours` / `.selected-day` | `HubView.weather.hours` / `.selected_day` (core đã lọc giờ theo ngày chọn) |
| `.days` / `.week-min` / `.week-max` | `HubView.weather.days` / `.week_min` / `.week_max` |
| `.focus` / `.focus-serial` | `HubView.weather.focus` / `.focus_serial` |
| `select-day(key)` | `UserInput::Focus(Some(Focus::Weather(key)))` |
| `WeatherHour.peak` | `HubView.weather.hours[].peak` |

## Answer

Implemented contract/weather.slint, screens/weather.slint, components/weather/{backdrop,current,hours-chart,days-column}.slint, Rust fixtures (fixtures/weather.rs + tests) and 11 demo scenarios. 24h chart y-scale uses week-min..week-max; peak marked via the selected day high. Pi fps per background not measured (ready-for-human). hub-core map left pending.

## Comments

- 2026-11-23 (review hub-ui): Điểm cao nhất do Rust chọn: thêm `WeatherHour.peak` (giờ sớm nhất nếu trùng), bỏ so sánh float `h.temp == day-high` trong `hours-chart.slint`. Có unit test. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
