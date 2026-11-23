# 03 — Bridge Đếm ngược và Reminder

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md) · UI: hub-ui #12, #13

## Việc

- `bridge/countdowns.rs`: `CountdownsData` từ `HubView.countdowns` (`main`, `has-main`, `others`, `confetti-serial`, `title`).
- `remove(key)` → `UserInput::RemoveCountdown(id)`.
- `select`, `open-detail`, `close-detail`: trạng thái trình bày do bridge giữ (mốc đang chọn, `detail-open`); áp lại khi `HubView.countdowns` đổi, bỏ chọn nếu mốc đã mất.
- `bridge/reminders.rs`: `RemindersData` từ `HubView.reminders`.
- `CountdownsView`, `RemindersView` chưa có `focus`/`focus-serial`: để mặc định. Ghi vào [docs/limitations.md](../../../docs/limitations.md) (đã có mục), không sửa hub-core trong ticket này.
- Điền bảng Map của hub-ui #12, #13.

## Nghiệm thu

- [ ] Tay: tạo mốc bằng giọng hoặc Setup → hiện ở `main`/`others`; xoá → mờ rồi mất, không còn sau khi khởi động lại.
- [ ] Tay: mốc tới ngày → confetti chạy một lần.
- [ ] Tay: Reminder hiện đúng nhóm "Hôm nay"/"Ngày mai", `next` đếm "còn N phút".
