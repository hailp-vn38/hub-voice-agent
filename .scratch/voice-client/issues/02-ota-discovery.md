# 02 — OTA lấy URL WS và token

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- `VoiceConfig.ota_url` là điểm vào duy nhất. Mỗi lần kết nối: gọi OTA với header `device-id`, `client-id`, đọc `websocket.url` và `websocket.token` từ `OtaResponse` (`crates/vela-server/src/app/enrollment/ota.rs`, `protocol/server.rs`), rồi mở WS như ticket 01.
- Kiểm request OTA (method, body, header) với code server và ghi lại vào `## Comments`.
- OTA trả lỗi HTTP (403 Device bị chặn, 503) hoặc thiếu `websocket` → `Connection(Offline { .. })`. Retry để ticket 04 xử lý.
- Server giả thêm endpoint OTA HTTP.

## Nghiệm thu

- [ ] Test: OTA trả URL + token → WS nhận đúng token và header → `Ready`.
- [ ] Test: OTA 403 → `Offline`, không mở WS.
- [ ] Test: OTA trả JSON thiếu `websocket` → `Offline`.
