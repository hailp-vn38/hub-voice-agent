# 02 — OTA lấy URL WS và token

Status: ready-for-human
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- `VoiceConfig.ota_url` là điểm vào duy nhất. Mỗi lần kết nối: gọi OTA với header `device-id`, `client-id`, đọc `websocket.url` và `websocket.token` từ `OtaResponse` (`crates/vela-server/src/app/enrollment/ota.rs`, `protocol/server.rs`), rồi mở WS như ticket 01.
- Kiểm request OTA (method, body, header) với code server và ghi lại vào `## Comments`.
- OTA trả lỗi HTTP (403 Device bị chặn, 503) hoặc thiếu `websocket` → `Connection(Offline { .. })`. Retry để ticket 04 xử lý.
- Server giả thêm endpoint OTA HTTP.

## Nghiệm thu

- [x] Test: OTA trả URL + token → WS nhận đúng token và header → `Ready`.
- [x] Test: OTA 403 → `Offline`, không mở WS.
- [x] Test: OTA trả JSON thiếu `websocket` → `Offline`.

## Comments

- Contract từ `crates/vela-server/src/app/mod.rs` và `app/enrollment/ota.rs`: route `/voice/ota` (có hoặc không có slash cuối) nhận GET và POST. GET không đọc body; POST đọc metadata JSON và yêu cầu `Content-Type: application/json`. Cả hai cần đúng một header `device-id` và `client-id`, mỗi giá trị hợp lệ dài tối đa 128 byte. `OtaResponse.websocket` là optional; khi có, `url` và `token` là chuỗi.
- Client dùng GET không body làm request OTA tối thiểu. Fake OTA xác nhận method, body rỗng, hai identity header; thành công chuyển URL/token và identity sang WS. Lỗi HTTP và thiếu `websocket` phát Offline.
- ADR-0073/0074 mô tả policy OTA và enrollment nhưng bỏ qua method, body và quy tắc header. Không thấy mâu thuẫn trực tiếp; đây là chi tiết source contract chưa được ghi trong ADR. `docs/codebase.md` §3.1 còn mô tả URL WS/token trong `ServerConfig`; spec ghi rõ sẽ cập nhật tài liệu đó ở ticket 08.
- Gate: `cargo fmt --all -- --check` PASS; `cargo test -p voice-client` PASS (5 passed); `cargo clippy -p voice-client --all-targets -- -D warnings` PASS; `scripts/test.sh -p voice-client` PASS (`ok: 6 passed`).
