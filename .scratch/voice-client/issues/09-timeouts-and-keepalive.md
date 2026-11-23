# 09 — Timeout cho OTA/connect/hello, keepalive sau Ready

Status: done
Spec: [../spec.md](../spec.md)
Nguồn: review nhánh feat/voice-client (2026-11-23)

## Việc

- `lib.rs` `discover_websocket`: tạo `reqwest::Client` có `timeout` (đặt tên hằng `OTA_TIMEOUT`, ~10 s).
- `connection.rs`: bọc `connect_async` và vòng chờ ServerHello bằng `tokio::time::timeout` (`HANDSHAKE_TIMEOUT`); hết giờ → `ConnectionEnd::Failed { became_ready: false }`.
- Chờ ServerHello: bỏ qua frame Ping/Pong thay vì coi là lỗi.
- Sau Ready: phát hiện socket half-open — idle timeout (không nhận frame nào trong N giây) hoặc gửi Ping định kỳ; hết giờ → `Failed { became_ready: true }` (vẫn phát `TtsStop{Interrupted}` trước `Offline` nếu đang có lượt TTS).

## Nghiệm thu

- [x] Test fake server nhận TCP/WS rồi im lặng → client phát `Offline` và thử lại (dùng `tokio::time::pause`).
- [x] Test Ping trước ServerHello → vẫn Ready.
- [x] Test im lặng sau Ready → `Offline`, có `TtsStop{Interrupted}` nếu đang phát.

## Comments

- 2026-11-23: Giới hạn gom vào `connection::Timeouts` (OTA 10 s, handshake 10 s, ping 15 s, idle 45 s), inject qua `spawn_with` giống `jitter`. Test paused-clock cũ dùng `Timeouts::NONE`, vì clock pause tự nhảy khi IO thật đang chờ và làm timeout nổ giả. Test mới ở `src/connection/timeout_tests.rs` chạy clock thật với giới hạn vài trăm ms. Chọn Ping định kỳ + idle timeout (frame bất kỳ, kể cả Pong, làm mới idle).
