# 04 — Reconnect có backoff, đổi config

Status: ready-for-human
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

- Mọi lần đứt (OTA lỗi, WS lỗi, server đóng, write timeout) → `Connection(Offline { retry_in })` rồi thử lại sau `retry_in`.
- Backoff: 1 s nhân đôi tới 60 s, jitter ±20 %, reset sau khi `Ready`. Jitter lấy từ nguồn ngẫu nhiên inject được để test xác định.
- Đóng `1000` sau Enrollment: kết nối lại ngay (ticket 03).
- `watch` của `VoiceConfig` đổi sang giá trị khác → đóng kết nối hiện tại (đóng `1000`) rồi kết nối lại ngay với config mới. Giá trị bằng nhau thì bỏ qua.
- Sender `VoiceCommand` bị drop hoặc receiver `VoiceEvent` bị drop → đóng socket, task kết thúc.

## Nghiệm thu

- [x] Test (`tokio::time::pause`): server từ chối liên tiếp → `retry_in` tăng 1, 2, 4… tới trần 60 s trong biên jitter.
- [x] Test: sau một lần `Ready` rồi đứt → `retry_in` quay về ~1 s.
- [x] Test: đổi `ota_url` → kết nối tới server giả thứ hai.
- [x] Test: gửi lại cùng giá trị config → không kết nối lại.
- [x] Test: drop cả hai đầu channel → task kết thúc, server giả thấy close.

## Comments

- Không phát hiện khác biệt giữa wire source của vela-server và ticket 04.
- Verification: `cargo test -p voice-client` (13 passed), `cargo fmt --check`, `cargo clippy -p voice-client --all-targets -- -D warnings`, `scripts/test.sh -p voice-client` (14 passed).
- Đổi config bằng giá trị khác khi đang kết nối đóng WebSocket hiện tại rồi chuyển ngay; config bằng nhau không ngắt attempt đang chạy. DirectSpeech nhận được trước Ready bị trả `Offline`, còn lệnh listen/audio trước Ready bị bỏ theo spec.
- ClientHello có write timeout 10 giây; đây là lựa chọn triển khai phía client, độc lập với giới hạn writer 10 giây của server. Helper riêng được để các ticket 05/06 tái sử dụng khi gửi lệnh.
