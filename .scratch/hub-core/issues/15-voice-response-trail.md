# 15 — Voice, chữ phản hồi, ToolTrail và chạm Orb

Status: ready-for-human
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #04, #05

## Việc

- `Conversation` ghi dòng chữ theo câu: speaker (người dùng / agent), mỗi câu mới (STT hoặc `sentence_start`) tăng serial dòng; `fading` khi lượt hội thoại kết thúc.
- `UiState` giữ danh sách bước tool của lượt hiện tại: `key`, nhãn, trạng thái chạy / xong / lỗi, từ `VoiceEvent::ToolCall` và `VoiceEvent::ServerTool`. Xoá khi lượt mới bắt đầu.
- Tăng `events.wake` khi vào Listening, `events.tool` khi có tool, `events.error` khi tool lỗi.
- `UserInput::OrbTapped`: Idle → bắt đầu lượt như `WakeWord`; đang trong lượt → kết thúc lượt.
- `present`: `voice` (state, `mic-muted` = false tới khi có G5), `response` (`words` đã tách từ, `speaker`, `line-serial`, `fading`), `tool_trail`.

## Nghiệm thu

- [x] Test qua `run()`: wake → STT → TTS hai câu → `response.line-serial` tăng 3 lần, speaker đúng.
- [x] Test: `ToolCall` thành công và lỗi → `tool_trail` có hai bước với trạng thái đúng; lượt mới xoá trail.
- [x] Test: `OrbTapped` khi Idle → `ListenStart` + mic bật.

## Comments

- `present` hiện cung cấp `voice`, `response` và `tool_trail`; chữ STT/TTS được tách theo khoảng trắng. `mic_muted` là `false` cho tới G5. Nguồn `TtsText` hiện không có metadata nhấn nên `Word.emphasis` là `false`.
- Đã chạy `scripts/test.sh -p hub-core` (119 passed), `cargo fmt --all -- --check`, `cargo check -p hub-core --all-targets` và `git diff --check` (đều thành công).
