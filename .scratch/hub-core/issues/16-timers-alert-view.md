# 16 — Hẹn giờ và lớp báo: dữ liệu hiển thị và thao tác chạm

Status: ready-for-agent
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #11, #14

## Việc

- Lưu tổng thời lượng của hẹn giờ (`duration_seconds`) — migration `0007_timer_duration.sql`, cập nhật `timers_load/save`. Không có field này thì `progress` không tính được.
- `UserInput::Timer(TimerInput)`: `QuickStart(minutes)`, `TogglePause(id)`, `AddMinute(id)`, `Cancel(id)`; dùng chung luật trong `rules.rs` với tool giọng nói.
- `present`: `timers` (`key`, `label`, `remaining` "6:42", `detail` "7 phút · xong lúc 14:32" / "Đã hết giờ lúc 14:32", `progress` 1→0, `phase` running / paused / final-seconds (≤10 s) / finished, `title`, `focus` + serial).
- `present`: `alert` (`kind`, `title`, `time`, `body`, `serial` = `events.alert`) từ `UiState.alert`.

## Nghiệm thu

- [ ] Test migration: DB cũ không có cột → hẹn giờ cũ tải được, `progress` dùng thời lượng còn lại làm tổng.
- [ ] Test qua `run()`: `QuickStart(5)` → 1 thẻ; `TogglePause` → `phase == paused`, `remaining` đứng yên khi tiến clock.
- [ ] Test `present`: còn 9 s → `final-seconds`; hết giờ → `finished` và `alert.serial` tăng.
