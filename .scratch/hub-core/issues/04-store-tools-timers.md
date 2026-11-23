# 04 — Store + khung tool `hub_*` + hẹn giờ

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- `src/store/`: `Store::open(dir)` / `Store::in_memory()`, SQLite theo ADR 0012, migration `0001_init.sql`; `store/timers.rs`.
- `src/tools/`: `definitions()` (serde + schemars từ `args.rs`), `parse(name, args) -> Result<ToolRequest, ToolError>`. `pub fn tool_definitions()` ở `lib.rs`; ticket 02 đưa nó vào `VoiceConfig`.
- `src/features/timers.rs`: ≤ 4 hẹn giờ, giờ kết thúc tuyệt đối, tạm dừng, `start/adjust/cancel/due(now)`.
- `runtime/tools.rs`: `VoiceEvent::ToolCall` → parse → feature → `Effect::Persist` + `Effect::Reply`. Tham số sai → reply lỗi ngay.
- Tool: `hub_timer_start`, `hub_timer_adjust`, `hub_timer_cancel`, `hub_timer_pause` (tên chốt theo concept.md).
- `UiState.timers`.

## Nghiệm thu

- [ ] Test qua `run()`: `ToolCall("hub_timer_start", {minutes:7, label:"luộc trứng"})` → reply OK, `UiState.timers` có một thẻ; khởi động lại `run` với cùng `Store` thì vẫn còn.
- [ ] Hẹn giờ thứ 5 → reply lỗi có thông điệp rõ.
- [ ] Unit test `timers` cho tạm dừng/tiếp tục và `due(now)`.
- [ ] `tool_definitions()` trả JSON Schema hợp lệ cho mọi tool.
