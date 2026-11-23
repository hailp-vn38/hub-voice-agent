# 06 — Alerts + đồng bộ giờ

Status: ready-for-agent
Blocked by: 03, 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `System` (bắt đầu với `time_synced() -> bool`); `Ports.system`. Adapter desktop trong `apps/hub` trả `true`; fake trong test.
- `src/alerts.rs`: hàng đợi báo toàn màn, lần lượt từng mục, tự tắt (hẹn giờ 2 phút), đợi `conversation.turn_active() == false`, giữ mục quá hạn khi `!time_synced()`. API `push`, `dismiss`, `tick(now, turn_active, synced)`.
- Hẹn giờ đến hạn → `alerts.push(Timer)` → `UiState.alert` + `Play(TimerChime)`; dismiss → `StopSound(TimerChime)`.
- `UserInput::DismissAlert`.

## Nghiệm thu

- [ ] Test codebase.md §6: hẹn giờ 7 phút, tiến clock 7 phút → `UiState.alert` hết giờ và có `Play(TimerChime)`.
- [ ] Đang trong lượt nói → alert chờ tới `TtsStop` rồi mới hiện.
- [ ] Hai hẹn giờ cùng hết → báo lần lượt.
- [ ] `time_synced == false` → không báo; chuyển `true` → báo.
- [ ] Không thao tác 2 phút → tự tắt.
