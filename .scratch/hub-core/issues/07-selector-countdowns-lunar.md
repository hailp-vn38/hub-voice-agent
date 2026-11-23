# 07 — Selector, mốc đếm ngược, âm lịch

Status: ready-for-human
Blocked by: 04
Spec: [../spec.md](../spec.md)

## Việc

- `src/selector.rs`: `select(items, query) -> Found / Ambiguous(Vec) / NotFound`; áp dụng lại cho tool hẹn giờ (huỷ/chỉnh theo nhãn).
- `src/lunar.rs`: thuật toán Hồ Ngọc Đức, UTC+7, `to_lunar(date)`, `to_solar(lunar_date)` (xử lý tháng nhuận).
- `src/features/countdowns/` (khuôn ticket 04): ≤ 8 mốc, `none` / `yearly` / `yearly_lunar`, `next_occurrence(now)`; `features/countdowns/store.rs`.
- Tool mốc đếm ngược + `UiState.countdowns`.

## Nghiệm thu

- [x] Unit test `lunar` với bảng mốc đã biết (Tết các năm 2020–2030, một năm có tháng nhuận).
- [x] `yearly_lunar` cho ngày 30 tháng thiếu → quy tắc rõ ràng, có test.
- [x] `selector`: khớp không dấu, nhiều kết quả → `Ambiguous`, reply tool liệt kê lựa chọn.
- [x] Test qua `run()`: tạo mốc → `UiState.countdowns` hiển thị số ngày đúng; qua nửa đêm thì cập nhật.

## Kiểm chứng

- Test lịch âm Tết 2020–2030 và tháng nhuận; năm 2030 dùng ngày 02/02 theo UTC+7.
- Test ngày 30 tháng thiếu chuyển thành ngày 29; ngày 29/2 dương lịch chuyển thành 28/2 năm không nhuận.
- Test qua `run()` tạo mốc, cập nhật sau nửa đêm, lưu lại sau khởi động, và trả danh sách ứng viên khi nhãn hẹn giờ mơ hồ.
- `hub_countdown_create` nhận `date` YYYY-MM-DD theo lịch của `repeat`, và `lunar_leap` khi cần tháng nhuận.

## Comments

- Review fix: `selector` ưu tiên nhãn khớp đúng (sau khi bỏ dấu) trước nhãn chỉ chứa chuỗi hỏi, trả chỉ số mục. Mốc `yearly_lunar` ở tháng nhuận rơi về tháng thường trong năm không có tháng nhuận đó; chỉ cần xét năm âm hiện tại và năm sau.
