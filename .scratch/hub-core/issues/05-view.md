# 05 — View: điều hướng, focus, tự về màn trung tâm

Status: needs-info
Blocked by: 04
Spec: [../spec.md](../spec.md)

## Việc

- `src/view.rs`: màn hiện tại, focus, tự về màn trung tâm sau ~60 s không thao tác, Ambient, "tạm xem". API `open(screen, focus)`, `on_user(..)`, `tick(now)`, trả `Vec<Effect>`.
- `view::screen_for_server_tool(name)`: `match` tên tool phía server → màn, cho `VoiceEvent::ServerTool` (không có module `server_tools` riêng).
- Tool `hub_*` mở màn liên quan (vd. hẹn giờ → màn Hẹn giờ, focus thẻ mới).
- `UserInput` điều hướng (chạm menu, quay lại). `UiState.view { screen, focus }`.

## Nghiệm thu

- [ ] Test: `hub_timer_start` → `UiState.view.screen == Timers` với focus thẻ mới; tiến clock 60 s không thao tác → `Center`.
- [x] Thao tác người dùng đặt lại bộ đếm tự quay về.
- [ ] `ServerTool{name, Started}` với tên đã biết → mở đúng màn.

## Comments

Điều hướng bằng `UserInput`, focus timer vừa tạo và thời hạn 60 giây đã có; đã kiểm tra focus khi tạo timer và thời hạn/reset qua `run`. Chưa có bảng tên tool server đáng tin cậy: vela-server dùng external MCP có tên theo cấu hình, còn `server_*_reminder` quản lý dữ liệu khác Reminder cục bộ (concept yêu cầu chặn). Cần xác nhận tên tool server thực tế trước khi ánh xạ. Ambient hiện có thể mở/thoát bằng input; thời gian tự vào Ambient và vị trí tạm xem chưa được đặc tả đầy đủ ở ticket này.
