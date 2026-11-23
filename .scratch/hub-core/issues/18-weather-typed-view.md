# 18 — Thời tiết: WeatherView có kiểu và dữ liệu hiển thị

Status: ready-for-agent
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #09

## Vì sao

`WeatherView.forecast: Option<serde_json::Value>` để định dạng JSON của Open-Meteo lọt qua `UiState`; mọi nơi đọc phải biết tên field của Open-Meteo.

## Việc

- Parse JSON trong feature weather (hoặc adapter `OpenMeteo`) thành kiểu: `Current`, `Vec<Hour>`, `Vec<Day>`, `AirQuality`. Cache trong store giữ nguyên JSON nếu cần, chỉ đổi phần đưa ra `UiState`.
- `present`: `weather` — `location`, `preview`, `updated` "HH:MM", `stale`, `background` (clear/cloudy/rain/storm/fog từ weather code), `is-day` (theo sunrise/sunset), `now` (feels, humidity, wind, uv, aqi + mức + màu), `hours` 24 giờ của ngày chọn có `peak`, `days`, `week-min/max`, `focus` + serial.
- `selected-day` do hub-core giữ (#14): suy từ `Focus::Weather` (key ngày; key giờ → ngày chứa giờ đó; `aqi` hoặc không có focus → hôm nay). Chạm `select-day` → `UserInput::Focus(Some(Focus::Weather(key)))`.

## Nghiệm thu

- [ ] Test parse với JSON mẫu Open-Meteo (fixture trong `tests/support/weather.rs`).
- [ ] Test `present`: giờ nóng nhất có `peak`; mã mưa → `background == rain`; dữ liệu cũ → `stale`.
- [ ] Test: focus key giờ của ngày mai → `selected-day` là ngày mai, `hours` là 24 giờ của ngày mai.

## Comments

- 2026-11-23: đã chốt ở #14, `selected-day` do hub-core giữ qua `Focus`.
