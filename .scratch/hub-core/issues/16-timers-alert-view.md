# 16 — Hẹn giờ và lớp báo: dữ liệu hiển thị và thao tác chạm

Status: completed
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #11, #14

## Việc

- Lưu tổng thời lượng của hẹn giờ (`duration_seconds`) — migration `0007_timer_duration.sql`, cập nhật `timers_load/save`. Không có field này thì `progress` không tính được.
- `UserInput::Timer(TimerInput)`: `QuickStart(minutes)`, `TogglePause(id)`, `AddMinute(id)`, `Cancel(id)`; dùng chung luật trong `rules.rs` với tool giọng nói.
- `present`: `timers` (`key`, `label`, `remaining` "6:42", `detail` "7 phút · xong lúc 14:32" / "Đã hết giờ lúc 14:32", `progress` 1→0, `phase` running / paused / final-seconds (≤10 s) / finished, `title`, `focus` + serial).
- `present`: `alert` (`kind`, `title`, `time`, `body`, `serial` = `events.alert`) từ `UiState.alert`.

## Nghiệm thu

- [x] Test migration: DB cũ không có cột → hẹn giờ cũ tải được, `progress` dùng thời lượng còn lại làm tổng.
- [x] Test qua `run()`: `QuickStart(5)` → 1 thẻ; `TogglePause` → `phase == paused`, `remaining` đứng yên khi tiến clock.
- [x] Test `present`: còn 9 s → `final-seconds`; hết giờ → `finished` và `alert.serial` tăng.

## Comments

- 2026-11-23: Thêm `duration_seconds` và migration `0007`; timer cũ được gán tổng thời lượng bằng thời gian còn lại theo `Clock` khi actor tải dữ liệu. `UserInput::Timer` xử lý bắt đầu nhanh, tạm dừng/tiếp tục, cộng một phút và huỷ; `present` dựng thẻ hẹn giờ và lớp báo từ `UiState`.
- Gate đã chạy: `scripts/test.sh -p hub-core` (119 passed), `cargo fmt --all -- --check`, `cargo build -p hub-core`, `cargo build -p hub`, `git diff --check`.
