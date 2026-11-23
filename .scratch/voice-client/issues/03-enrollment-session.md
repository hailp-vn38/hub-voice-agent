# 03 — Đăng ký Device qua OTA và mã kích hoạt

Status: ready-for-human
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

Hub dùng chế độ đăng ký qua OTA của vela-server (`devices.enrollment.transport = "ota"`). Device chưa được link thì không mở WebSocket: OTA trả mã kích hoạt, người dùng nhập mã vào admin web để map Device vào agent, người dùng bấm "Kết nối" lại để vào voice.

Contract server (repo `AI_Agent_Voice_rust`):

- `GET /voice/ota` (`crates/vela-server/src/app/enrollment/ota.rs:27-112`, schema `protocol/server.rs:37-70`):
  - Device chưa link → `activation { code, message, challenge, timeout_ms }`, **không có** `websocket`.
  - Device đã đăng ký → có `websocket { url, token }`, không có `activation`.
  - Lỗi: `403 device_not_admitted`, `503` (`database_unavailable`, `server_shutting_down`, `enrollment_capacity_exceeded`, `enrollment_code_unavailable`).

Không dùng `POST /voice/ota/activate`: endpoint này chỉ báo trạng thái chờ cho firmware tương thích. Gọi lại OTA là đủ, vì OTA trả lại mã đang chờ (`get_or_create_enrollment`) khi chưa link và trả `websocket` khi đã link.

Phía `voice-client` (`enrollment.rs`):

- `spawn` tự gọi OTA và kết nối như ticket 02; lỗi OTA/WS retry theo backoff (ticket 04).
- OTA trả `websocket` → mở WS → `Ready`.
- OTA trả `activation` → phát `Connection(Enrolling)` rồi `Activation { code }` lấy nguyên từ `activation.code`. Không parse text, không tự gọi lại OTA, không backoff. Người dùng nhập mã trên admin web rồi bấm "Kết nối" trong Settings → `VoiceCommand::Connect` → gọi OTA một lần.
- `Connect` chỉ có tác dụng khi đang `Enrolling`; trạng thái khác thì bỏ qua.
- Trong lúc `Enrolling`, `ListenStart`/`ListenStop`/`Abort`/`Audio`/`DirectSpeech` xử lý như khi chưa Ready (bất biến 2): `DirectSpeech` → `DirectSpeechRejected { reason: Offline }`.
- OTA trả cả `websocket` lẫn `activation` → ưu tiên `websocket`. OTA không trả cả hai → `Offline` như ticket 02.
- Đổi `VoiceConfig` khi đang `Enrolling` → gọi OTA với config mới (bất biến 6).

Chế độ đăng ký qua WebSocket (`transport = "websocket"`, mặc định của server, ADR 0074) không được hỗ trợ: Device chưa link sẽ nhận `websocket` và vào Enrollment Session, mà client không phân biệt được với Voice Session. Ghi yêu cầu `transport = "ota"` vào tài liệu triển khai hub.

Server giả OTA trả kịch bản theo lượt: `activation`, rồi `websocket`.

## Nghiệm thu

- [x] Test: OTA trả `activation` → `Enrolling`, `Activation { code }` đúng mã, không mở WS, không gọi OTA lần hai (`tokio::time::pause`, chờ 60 s).
- [x] Test: đang `Enrolling`, gửi `Connect`, OTA trả `websocket` → `Ready`.
- [x] Test: đang `Enrolling`, gửi `Connect`, OTA vẫn trả `activation` → vẫn `Enrolling`, phát lại `Activation { code }`.
- [x] Test: `Connect` khi `Ready` → không gọi OTA.
- [x] Test: `DirectSpeech` khi `Enrolling` → `DirectSpeechRejected { reason: Offline }`.
- [x] Test: đổi `VoiceConfig` khi `Enrolling` → gọi OTA với config mới.

## Comments

Bản trước của ticket theo Enrollment Session qua WebSocket (ADR 0074) và bị chặn ở `needs-info`: ServerHello không có trường phân biệt loại session, mã chỉ nằm trong `stt.text`, và close `1000` không phân biệt claim với timeout. Đã chuyển sang chế độ OTA, server có sẵn, không cần đổi wire contract.

- Đã triển khai OTA activation và `Connect`; Settings và kiểm thử với vela-server thật thuộc các bước tiếp theo. Kiểm tra: `cargo check --workspace`, `cargo clippy -p voice-client --all-targets -- -D warnings`, `scripts/test.sh` (51 test) đều đạt.
