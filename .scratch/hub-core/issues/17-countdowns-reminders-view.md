# 17 — Đếm ngược và Reminder: dữ liệu hiển thị

Status: ready-for-human
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #12, #13

## Việc

- `present`: `countdowns` — `main` là mốc gần nhất chưa qua, `others`; `unit` days / live (còn < 1 ngày, "HH:MM:SS") / today / passed; `target` dạng tiếng Việt; `lunar` ("15/8 âm lịch" hoặc ""); `repeat-text`; `progress`; `confetti-serial` khi một mốc tới ngày.
- `UserInput::RemoveCountdown(id)`; `fading` do UI tự làm hay hub-core giữ một nhịp trước khi gỡ — chốt khi làm, ghi vào Comments.
- `present`: `reminders` — `title` "N sắp tới", `next` (`until` "còn 25 phút"), `groups` theo ngày ("Hôm nay", "Ngày mai", "Thứ Sáu 28/11"), `recent` từ `reminder_history` với `missed`.

## Nghiệm thu

- [x] Test `present` với `now` cố định: mốc hôm nay → `unit == today`; mốc âm lịch có `lunar`; mốc còn 3 giờ → `live`.
- [x] Test: Reminder hằng ngày và một lần → nhóm và `repeat-text` đúng; `next.until` đổi theo `now`.
- [x] Test qua `run()`: `RemoveCountdown` → mục biến mất khỏi store và `UiState`.

## Comments

- 2026-11-23: `RemoveCountdown(id)` xoá ngay khỏi Store và `UiState`. Theo ADR 0002, UI sở hữu animation mờ dần; hub-core không giữ mốc thêm một nhịp. Khi xoá mốc đang focus, focus cũng được gỡ.
- 2026-11-23: Đã thêm ngày tạo mốc để tính `progress`; bản ghi cũ không có metadata dùng tiến độ 0 cho mốc tương lai. Snapshot đầu khởi tạo `UiState.now` từ `Clock`, để presenter có giờ đúng ngay khi chạy.
- 2026-11-23: Sau khi merge integration mới nhất, gate tự động: `scripts/test.sh -p hub-core` (126 passed), `cargo fmt --all -- --check`, `cargo build -p hub`, `git diff --check` đều qua. Kiểm tra map/hiển thị Slint thuộc hub-ui #12/#13, chưa nằm trong gate của ticket này.
