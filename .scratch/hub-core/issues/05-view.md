# 05 — View: điều hướng, focus, tự về màn trung tâm

Status: ready-for-agent
Blocked by: 04
Spec: [../spec.md](../spec.md)

## Việc

- `src/view.rs`: màn hiện tại, focus, tự về màn trung tâm sau ~60 s không thao tác, Ambient, "tạm xem". API `open(screen, focus)`, `on_user(..)`, `tick(now)`, trả `Vec<Effect>`.
- `view::screen_for_server_tool(name)`: `match` tên tool phía server → màn, cho `VoiceEvent::ServerTool` (không có module `server_tools` riêng).
- Tool `hub_*` mở màn liên quan (vd. hẹn giờ → màn Hẹn giờ, focus thẻ mới).
- `UserInput` điều hướng (chạm menu, quay lại). `UiState.view { screen, focus }`.

## Nghiệm thu

- [ ] Test: `hub_timer_start` → `UiState.view.screen == Timers` với focus thẻ mới; tiến clock 60 s không thao tác → `Center`.
- [ ] Thao tác người dùng đặt lại bộ đếm tự quay về.
- [ ] `ServerTool{name, Started}` với tên đã biết → mở đúng màn.
