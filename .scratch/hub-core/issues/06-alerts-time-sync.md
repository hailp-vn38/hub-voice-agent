# 06 — Alerts + đồng bộ giờ

Status: ready-for-human
Blocked by: 03, 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `System` (bắt đầu với `time_synced() -> bool`); `Ports.system`. Adapter desktop trong `apps/hub` trả `true`; fake trong test.
- `src/alerts.rs`: hàng đợi báo toàn màn, lần lượt từng mục, tự tắt (hẹn giờ 2 phút), đợi `conversation.turn_active() == false`, giữ mục quá hạn khi `!time_synced()`. API `push`, `dismiss`, `tick(now, turn_active, synced)`.
- Hẹn giờ đến hạn → `alerts.push(Timer)` → `UiState.alert` + `Play(TimerChime)`; dismiss → `StopSound(TimerChime)`.
- `UserInput::DismissAlert`.

## Nghiệm thu

- [x] Test codebase.md §6: hẹn giờ 7 phút, tiến clock 7 phút → `UiState.alert` hết giờ và có `Play(TimerChime)`.
- [ ] Đang trong lượt nói → alert chờ tới `TtsStop` rồi mới hiện.
- [x] Hai hẹn giờ cùng hết → báo lần lượt.
- [x] `time_synced == false` → không báo; chuyển `true` → báo.
- [x] Không thao tác 2 phút → tự tắt.

## Comments

Đã kiểm thử qua `run` với clock giả: hết giờ, hàng đợi, chưa đồng bộ, tự tắt và dismiss. Chưa có test qua `run` cho trường hợp đợi `TtsStop`; logic dùng `conversation.turn_active()` trên tick. Adapter NTP Pi chưa triển khai (ngoài phạm vi spec); placeholder Pi trả `false`, desktop trả `true`. Audio desktop hiện chỉ tiêu thụ lệnh, chưa phát âm thanh thật.
