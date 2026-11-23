# 09 — Thời tiết (Open-Meteo)

Status: completed
Blocked by: 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `WeatherSource`; adapter `sources/open_meteo.rs` (forecast, air quality, geocoding); fake trong `tests/support`; fixture JSON thật ở `tests/fixtures/`.
- `src/features/weather/` (khuôn ticket 04): vị trí nhà, thành phố tạm xem (giữ ~30 phút), WMO → icon/nền, `snapshot(focus)`, `apply(forecast)`; cache trong `Store` để hiển thị khi offline.
- Làm mới 30 phút trên tick của actor (`Effect::Spawn`, không spawn vòng riêng). Tool thời tiết ở thành phố khác: `Effect::Spawn` kèm `ToolCall`, khoá job theo tính năng để câu hỏi mới huỷ câu cũ; `JobDone` → cập nhật state rồi mới reply.

## Nghiệm thu

- [x] Adapter parse được fixture; không gọi mạng trong test.
- [x] Test qua `run()`: tool hỏi thời tiết Đà Nẵng → mở màn Thời tiết ở chế độ tạm xem, reply có nhiệt độ; rời màn → quay về vị trí nhà ngay, cache giữ 30 phút.
- [x] Nguồn lỗi → giữ dữ liệu cache, `UiState` đánh dấu cũ.

## Comments

- Đã thêm Open-Meteo, cache SQLite, refresh theo tick và job tool; test adapter dùng fixture API, test actor dùng fake không gọi mạng.

- Review fix: rời màn tạm xem khôi phục hạn làm mới theo cache của vị trí nhà (30 phút sau lần tải), không tải lại ngay khi cache còn mới.
