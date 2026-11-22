# 01 — Handshake tới Ready qua server giả

Status: ready-for-agent
Blocked by: —
Spec: [../spec.md](../spec.md)

## Việc

Tracer bullet đi qua cả interface công khai:

- Khai báo đầy đủ type công khai trong `lib.rs` theo spec (`VoiceConfig`, `VoiceCommand`, `VoiceEvent`, `ToolCall`…). Biến thể chưa làm thì dùng `todo!()` ở chỗ xử lý, không ở type.
- `protocol.rs` (private): ClientHello V1 (`version: 1`, `transport: "websocket"`, uplink Opus 16 kHz mono 60 ms, features `mcp`, `tool_status`, `direct_tts`, `aec` = true) và ServerHello (`session_id`, downlink audio_params). Đối chiếu với `crates/vela-server/src/protocol/`.
- `connection.rs`: mở WS với header `Authorization: Bearer <token>`, `device-id`, `client-id`, gửi hello, chờ ServerHello, phát `Connection(Connecting)` rồi `Connection(Ready)`.
- Ở ticket này `spawn` nhận thẳng URL WS + token qua một hàm nội bộ. Ticket 02 thay bằng OTA.
- `tests/support/fake_server.rs`: server WS giả trên `127.0.0.1:0`, chạy kịch bản (nhận frame → kiểm → gửi frame), ghi lại header và frame đã nhận.

## Nghiệm thu

- [ ] Test: server giả kiểm header và ClientHello đúng schema, trả ServerHello → nhận `Connecting` rồi `Ready`.
- [ ] Test: server đóng `1002` sau hello → nhận `Connection(Offline { .. })`, không panic.
- [ ] Mọi `session_id` gửi đi khớp `session_id` trong ServerHello.
- [ ] `protocol.rs` không lộ type `pub` ra ngoài crate.
