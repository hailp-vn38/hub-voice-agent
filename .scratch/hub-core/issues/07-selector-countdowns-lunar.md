# 07 — Selector, mốc đếm ngược, âm lịch

Status: ready-for-agent
Blocked by: 04
Spec: [../spec.md](../spec.md)

## Việc

- `src/selector.rs`: `select(items, query) -> Found / Ambiguous(Vec) / NotFound`; áp dụng lại cho tool hẹn giờ (huỷ/chỉnh theo nhãn).
- `src/lunar.rs`: thuật toán Hồ Ngọc Đức, UTC+7, `to_lunar(date)`, `to_solar(lunar_date)` (xử lý tháng nhuận).
- `src/features/countdowns/` (khuôn ticket 04): ≤ 8 mốc, `none` / `yearly` / `yearly_lunar`, `next_occurrence(now)`; `features/countdowns/store.rs`.
- Tool mốc đếm ngược + `UiState.countdowns`.

## Nghiệm thu

- [ ] Unit test `lunar` với bảng mốc đã biết (Tết các năm 2020–2030, một năm có tháng nhuận).
- [ ] `yearly_lunar` cho ngày 30 tháng thiếu → quy tắc rõ ràng, có test.
- [ ] `selector`: khớp không dấu, nhiều kết quả → `Ambiguous`, reply tool liệt kê lựa chọn.
- [ ] Test qua `run()`: tạo mốc → `UiState.countdowns` hiển thị số ngày đúng; qua nửa đêm thì cập nhật.
