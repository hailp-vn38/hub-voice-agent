# 12 — Dọn code theo review: trùng lặp, protocol, layout test, docs

Status: done
Blocked by: 09, 10, 11
Spec: [../spec.md](../spec.md)
Nguồn: review nhánh feat/voice-client (2026-11-23)

## Việc

Không đổi hành vi; mọi test hiện có phải giữ xanh trước và sau.

- Layout test (AGENTS.md): mỗi module tự khai báo `#[cfg(test)] #[path = "x_tests.rs"] mod tests;` — `connection_tests` dưới `connection.rs`, `direct_speech_tests` dưới `connection/voice.rs`, `mcp_tests` dưới `mcp.rs`. Tách `mcp_tests.rs` (333 dòng) thành thư mục `mcp_tests/`.
- `protocol.rs`: chuyển việc dựng frame `listen`/`abort`/`direct_speech`/`mcp` và `parse_text_event` từ `voice.rs` sang đây; dùng enum serde cho `source`/`state`/`outcome`/`reason` thay vì so chuỗi. Parse JSON một lần mỗi frame.
- `voice.rs`: gom 7 chỗ `send_with_timeout … interrupted_stop … Failed` vào một helper; tách `run` (~140 dòng) thành handler theo nguồn (socket / lệnh / config). Xoá nhánh `TtsAudio` chết.
- `connection.rs`: gom 3 khối `HeaderValue::from_str` vào một hàm; gom arm config-changed trùng giữa `await_operation` và `voice::run`. Header không hợp lệ → báo lỗi rõ thay vì `Failed` lặp vô hạn.
- `ClientIo`: dùng nhất quán (`lib.rs:58`, `:115` đang truyền tay 5 tham số); `active_config` thành `&`.
- Thay `Err(())` bằng enum lỗi nội bộ và `tracing` log nguyên nhân reconnect.
- Đặt tên hằng cho 128 / 1_024 / 4_096 trong `validate_direct_speech`, dẫn chiếu spec.
- Cân nhắc tách OTA discovery + backoff khỏi `lib.rs` (`ota.rs`, `reconnect.rs`).
- `docs/codebase.md` §2/§3.1: thêm `connection/voice.rs`, `reconnect_tests/`, file mới; đánh dấu `enrollment.rs` là chưa có (issue 03).

## Nghiệm thu

- [x] `scripts/test.sh -p voice-client`, clippy `-D warnings`, fmt qua.
- [x] Không file nguồn/test nào > ~300 dòng.
- [x] `rg 'json!\(' crates/voice-client/src/connection` không còn dựng frame wire.

## Comments

- 2026-11-23: `ClientIo` + `await_operation` + `Operation` thay bằng `connection::Client` sở hữu channel, config và `Timeouts`; `client.operation(..)` trả `Result<T, ConnectionEnd>`. `Err(())` thay bằng `connection::Failure` (Display), reconnect loop `log::warn!` nguyên nhân. Header không hợp lệ giờ là `Failure::InvalidRequest` trong log; vẫn retry theo backoff vì config có thể được sửa. `lib.rs` tách thành `reconnect.rs` + `ota.rs`; `Timeouts` sang `connection/timeouts.rs`. Wire format (frame uplink, parse downlink bằng enum serde, kiểm direct speech với hằng đặt tên) gom vào `protocol.rs`; `mcp::tool_status` gộp vào `protocol::downlink_event`. Vòng Ready tách thành `Session` với handler `on_frame`/`on_text`/`on_command`/`finish`. Test: `connection_tests.rs` khai báo trong `connection.rs`, test direct speech chuyển sang `connection/voice_tests.rs`, `mcp_tests/` (mod + discovery/calls/edges), `reconnect_tests/` khai báo trong `reconnect.rs`, thêm `protocol_tests.rs`. `ConnectionEnd::Reconfigured` box config (clippy `result_large_err`).
