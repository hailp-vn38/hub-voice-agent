# 06 — Bridge Trung tâm, Menu và Ambient

Status: ready-for-human
Blocked by: 02, 03, 04, 05
Spec: [../spec.md](../spec.md) · UI: hub-ui #07, #08, #18

## Việc

- `bridge/center.rs`:
  - `Center` từ `HubView.center`; `open(screen)` → `UserInput::Open`; `toggle-home(key)` → `ToggleHome`.
  - `Menu` từ `HubView.menu`.
  - `AmbientData` từ `HubView.ambient`; `wake` → `UserInput::Activity`.
- Kiểm handler `changed focus-serial` của `screens/center.slint` bỏ qua khi `focus` là `none`.
- Điền bảng Map của hub-ui #07, #08, #18.

## Nghiệm thu

- [ ] Tay: màn trung tâm có giờ, thời tiết, sự kiện hôm nay, Reminder, hẹn giờ gần hết nhất, mốc gần nhất, tối đa 4 thiết bị nhà.
- [ ] Tay: dòng Menu đổi theo dữ liệu (vd. tạo hẹn giờ → dòng Hẹn giờ đổi).
- [ ] Tay: để yên 5 phút → Ambient; bấm → về trung tâm.
- [ ] Tay: chạy app thật 30 phút không có cảnh báo hay lỗi trong log.

## Comments

- 2026-11-23: Đã viết bridge và test hợp đồng/callback. `scripts/test.sh -p hub` qua với 168 test. Chưa chạy UI trên Mac hoặc app thật 30 phút, vì vậy các gate thủ công vẫn để trống.
