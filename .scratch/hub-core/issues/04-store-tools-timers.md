# 04 — Store + mẫu thư mục tính năng + hẹn giờ

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

Ticket này dựng **khuôn thư mục tính năng** (spec, thay đổi 4) mà ticket 07–12 làm theo.

- `src/store/mod.rs`: `Store::open(dir)` / `Store::in_memory()`, SQLite theo ADR 0012, migration `0001_init.sql`. Chỉ mở kết nối và chạy migration.
- `src/features/timers/`:
  - `rules.rs`: luật thuần, ≤ 4 hẹn giờ, giờ kết thúc tuyệt đối, tạm dừng, `start/adjust/cancel/due(now)` trả kết quả nghiệp vụ, không trả `Effect`.
  - `store.rs`: `impl Store { timers_* }`.
  - `tools.rs`: struct tham số (serde + schemars), `definitions()`, `parse(name, args) -> Result<TimerTool, ToolError>`.
  - `mod.rs`: điều phối, `on_tool(..)`, `tick(now)`, `snapshot()` → `Vec<Effect>` (`Effect::Persist`, `Effect::Reply`, `Effect::Ui`).
- `runtime/tools.rs`: định tuyến theo tiền tố tên (`hub_timer_*` → `timers`); tên lạ → reply lỗi. Tham số sai → reply lỗi ngay.
- `pub fn tool_definitions()` ở `lib.rs` nối `definitions()` của từng tính năng; ticket 02 đưa nó vào `VoiceConfig`.
- Tool: `hub_timer_start`, `hub_timer_adjust`, `hub_timer_cancel`, `hub_timer_list` (tên chốt theo concept.md).
- `UiState.timers`.

## Nghiệm thu

- [ ] Test qua `run()`: `ToolCall("hub_timer_start", {minutes:7, label:"luộc trứng"})` → reply OK, `UiState.timers` có một thẻ; khởi động lại `run` với cùng `Store` thì vẫn còn.
- [ ] Hẹn giờ thứ 5 → reply lỗi có thông điệp rõ.
- [ ] Unit test `timers/rules.rs` cho tạm dừng/tiếp tục và `due(now)`, không cần actor hay `Effect`.
- [ ] `tool_definitions()` trả JSON Schema hợp lệ cho mọi tool.
