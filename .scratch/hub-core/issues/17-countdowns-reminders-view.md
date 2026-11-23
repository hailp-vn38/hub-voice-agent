# 17 — Đếm ngược và Reminder: dữ liệu hiển thị

Status: ready-for-agent
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #12, #13

## Việc

- `present`: `countdowns` — `main` là mốc gần nhất chưa qua, `others`; `unit` days / live (còn < 1 ngày, "HH:MM:SS") / today / passed; `target` dạng tiếng Việt; `lunar` ("15/8 âm lịch" hoặc ""); `repeat-text`; `progress`; `confetti-serial` khi một mốc tới ngày.
- `UserInput::RemoveCountdown(id)`; `fading` do UI tự làm hay hub-core giữ một nhịp trước khi gỡ — chốt khi làm, ghi vào Comments.
- `present`: `reminders` — `title` "N sắp tới", `next` (`until` "còn 25 phút"), `groups` theo ngày ("Hôm nay", "Ngày mai", "Thứ Sáu 28/11"), `recent` từ `reminder_history` với `missed`.

## Nghiệm thu

- [ ] Test `present` với `now` cố định: mốc hôm nay → `unit == today`; mốc âm lịch có `lunar`; mốc còn 3 giờ → `live`.
- [ ] Test: Reminder hằng ngày và một lần → nhóm và `repeat-text` đúng; `next.until` đổi theo `now`.
- [ ] Test qua `run()`: `RemoveCountdown` → mục biến mất khỏi store và `UiState`.
