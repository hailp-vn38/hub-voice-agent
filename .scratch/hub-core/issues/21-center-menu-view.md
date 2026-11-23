# 21 — Màn trung tâm và Menu: dữ liệu gộp nhiều tính năng

Status: ready-for-agent
Blocked by: 16, 17, 18, 19, 20
Spec: [../spec.md](../spec.md) · UI: hub-ui #07, #08

## Việc

- `present`: `center` — `time`, `date` (thứ, ngày, âm lịch), `weather` tóm tắt, `today` (sự kiện hôm nay, `now`), `reminders` sắp tới, `timer` (hẹn giờ gần hết nhất), `countdown` (mốc gần nhất), `home` (tối đa 4 entity), `home-safe`, `focus: CenterCard` + serial.
- `present`: `menu` — mỗi tính năng một dòng thông tin sống ("2 hẹn giờ đang chạy", "28°, mưa nhẹ"…).
- `Center.toggle-home` dùng lại `UserInput::ToggleHome` (#20). Chọn 4 entity nào là G6 (Setting Nhà); tới khi có G6 lấy 4 entity đầu.

## Nghiệm thu

- [ ] Test `present` với `UiState` đầy đủ: center chọn đúng hẹn giờ gần hết nhất và mốc gần nhất; không có dữ liệu → các thẻ ẩn (`visible == false`).
- [ ] Test `menu`: mỗi dòng đổi khi dữ liệu tính năng đổi.
