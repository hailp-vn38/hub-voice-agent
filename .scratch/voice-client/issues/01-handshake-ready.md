# 01 — Handshake tới Ready qua server giả

Status: ready-for-human
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

- [x] Test: server giả kiểm header và ClientHello đúng schema, trả ServerHello → nhận `Connecting` rồi `Ready`.
- [x] Test: server đóng `1002` sau hello → nhận `Connection(Offline { .. })`, không panic.
- [x] Ticket này chưa gửi message nào mang `session_id`; protocol giữ ID từ ServerHello để các ticket xử lý lệnh dùng về sau.
- [x] `protocol.rs` không lộ type `pub` ra ngoài crate.

## Comments

- Đối chiếu code `crates/vela-server/src/protocol/client.rs` và `server.rs`: schema ClientHello/ServerHello cùng audio profile khớp ticket. Ví dụ ClientHello trong `docs/flows/01-websocket.md` để `mcp`, `tool_status`, `direct_tts`, `aec` false hoặc vắng trường; parser server hỗ trợ các feature này, và ticket/spec yêu cầu gửi cả bốn là `true`, nên client gửi `true`.
- Test handshake gọi seam WS nội bộ với URL + token như ticket quy định, nhận sự kiện qua channel và kiểu công khai. `spawn(VoiceConfig)` giữ interface hoàn chỉnh; ticket 02 sẽ nối entry point này qua OTA. Không đưa OTA vào ticket 01.
- Gate hoàn tất: hai test `voice-client`, fmt và clippy `-D warnings` đều qua. `scripts/agent-test.sh` không có trong repo; `scripts/test.sh -p voice-client` qua.
