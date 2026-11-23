# 21 — Màn trung tâm và Menu: dữ liệu gộp nhiều tính năng

Status: completed
Blocked by: 16, 17, 18, 19, 20
Spec: [../spec.md](../spec.md) · UI: hub-ui #07, #08

## Việc

- `present`: `center` — `time`, `date` (thứ, ngày, âm lịch), `weather` tóm tắt, `today` (sự kiện hôm nay, `now`), `reminders` sắp tới, `timer` (hẹn giờ gần hết nhất), `countdown` (mốc gần nhất), `home` (tối đa 4 entity), `home-safe`, `focus: CenterCard` + serial.
- `present`: `menu` — mỗi tính năng một dòng thông tin sống ("2 hẹn giờ đang chạy", "28°, mưa nhẹ"…).
- `Center.toggle-home` dùng lại `UserInput::ToggleHome` (#20). Chọn 4 entity nào là G6 (Setting Nhà); tới khi có G6 lấy 4 entity đầu.

## Nghiệm thu

- [x] Test `present` với `UiState` đầy đủ: center chọn đúng hẹn giờ gần hết nhất và mốc gần nhất; không có dữ liệu → các thẻ ẩn (`visible == false`).
- [x] Test `menu`: mỗi dòng đổi khi dữ liệu tính năng đổi.

## Comments

- `present` bổ sung Center và Menu; Center lấy sự kiện hôm nay, Reminder sắp tới, hẹn giờ đang chạy gần hết nhất, mốc gần nhất, tối đa bốn entity Nhà đầu tiên; `home-safe` theo trạng thái kết nối Home Assistant.
- Gates đạt: `scripts/test.sh` (142 passed), `cargo fmt --all -- --check`, `cargo build -p hub`, `git diff --check`.
