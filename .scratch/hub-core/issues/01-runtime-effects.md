# 01 — Khung runtime: Effect, executor, jobs, harness test

Status: ready-for-human
Blocked by: —
Spec: [../spec.md](../spec.md)

## Việc

- Chuyển actor sang `src/runtime/`: `mod.rs` (vòng `select!` qua tick và channel), `effects.rs` (`enum Effect` + executor duy nhất), `tick.rs` (đồng hồ hiện có).
- `Effect` ban đầu: `Ui(fn/patch cập nhật UiState)`, `Voice(VoiceCommand)`, `Audio(AudioCommand)`, `Spawn { job, work, reply, timeout }`. Các ticket sau thêm biến thể.
- `runtime/jobs.rs` (spec, thay đổi 5): giữ job đang chạy theo `JobId`, kết quả quay về vòng `select!` thành `Internal::JobDone`; hết `timeout` → reply lỗi cho `ToolCall` đi kèm; job mới cùng khoá huỷ job cũ.
- Thêm vào `Io`: `voice_tx`, `voice_rx`, `input_rx` (chưa xử lý gì, chỉ nối vào `select!`). `run` kết thúc khi `ui_tx` đóng như hiện nay.
- `tests/support`: harness `Hub::start()` dựng `run()` với mọi channel, `TestClock`, và trả các đầu channel cho test.

## Nghiệm thu

- [x] Test `clock` hiện có vẫn qua, viết lại bằng harness.
- [x] Unit test executor: một `Effect::Voice` → command xuất hiện trên `voice_tx`.
- [x] Test `jobs`: job xong → `JobDone` đúng id; quá `timeout` → `ToolCall` nhận reply lỗi; job mới cùng khoá → kết quả job cũ bị bỏ.
- [ ] `apps/hub` vẫn build và hiển thị đồng hồ (`cargo build -p hub`).
- [x] Hành vi không đổi; code di chuyển và tách ở commit riêng.

## Comments

- `cargo test -p hub-core`: 4 unit test và 1 integration test qua; test job xác minh hoàn thành, timeout và loại kết quả cũ đã xếp hàng. Đường `ToolCall::reply` khi timeout chưa được kiểm thử với ToolCall thật: type này không có constructor công khai; fake server của voice-client cần dựng thêm kết nối và protocol.
- `cargo build -p hub`: qua; chưa kiểm tra ứng dụng hiển thị đồng hồ thực tế. Hai điều kiện còn mở cần xác minh trước khi đóng ticket.
- Commit tách actor riêng: `82279a5`; clock test qua trước và sau khi chuyển.

- 2026-11-23: Integration test qua `run()` và voice-client thật với OTA/WebSocket giả xác minh Device MCP tool tạo lịch bị timeout nhận lỗi JSON-RPC `-32603`, `Job timed out`; clock ảo chỉ được tiến sau khi nguồn lịch bắt đầu. Hiển thị đồng hồ thực tế vẫn chờ kiểm tra thủ công.
