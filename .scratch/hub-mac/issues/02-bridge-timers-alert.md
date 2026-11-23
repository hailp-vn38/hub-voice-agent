# 02 — Bridge Hẹn giờ và lớp báo

Status: ready-for-human
Blocked by: 01
Spec: [../spec.md](../spec.md) · UI: hub-ui #11, #14

## Việc

- `bridge/timers.rs`: `TimersData` từ `HubView.timers` (`title`, thẻ, `focus`, `focus-serial` = `serial`).
- Callback → `UserInput::Timer(..)`: `quick-start(min)` → `QuickStart`, `toggle-pause(key)` → `TogglePause`, `add-minute(key)` → `AddMinute`, `cancel(key)` → `Cancel`. `key` là id dạng chuỗi; không parse được thì log và bỏ qua.
- `AlertData` từ `HubView.alert` (`None` → `kind: none`); `dismiss` → `UserInput::DismissAlert`.
- Kiểm `timer-card.slint` chỉ phát sáng khi thẻ đúng là focus (đã có `if (root.focused)`).
- Điền bảng Map của hub-ui #11, #14 (`VoiceSlider` để trống: chờ G3).

## Nghiệm thu

- [ ] Tay: bấm bắt đầu nhanh 1 phút → thẻ chạy, vòng giảm mượt; tạm dừng/tiếp tục; +1 phút; huỷ.
- [ ] Tay: hết giờ → lớp báo hiện; bấm tắt → lớp báo biến mất. (Chuông kiểm ở #07.)
- [ ] Tay (giọng, sau #08): "đặt hẹn giờ 2 phút" → mở màn Hẹn giờ, thẻ mới phát sáng.

## Comments

- 2026-11-23: Bridge hoàn tất ở commit `93214e4`; `scripts/test.sh -p hub --no-default-features --features desktop` đạt 154 passed. Các bước kiểm tay trên Mac còn chờ người dùng, vì vậy giữ nguyên ô nghiệm thu chưa tick.
