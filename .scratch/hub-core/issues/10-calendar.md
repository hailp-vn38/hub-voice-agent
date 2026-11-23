# 10 — Lịch Google

Status: completed
Blocked by: 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `CalendarSource`; adapter `sources/google_calendar.rs` (events.list với `syncToken`, events.insert, OAuth đổi mã + refresh theo ADR 0007). Refresh token lưu trong `ConfigStore`.
- `src/features/calendar/` (khuôn ticket 04): cache ~60 ngày, `apply(delta)`, `events(range)`, ghi nhớ sự kiện mới tạo.
- Làm mới 5 phút trên tick của actor qua `Effect::Spawn`; tạo sự kiện cũng qua `Effect::Spawn` kèm `ToolCall`; `syncToken` hết hạn (410) → đồng bộ lại toàn bộ.
- Tool xem/tạo sự kiện.

## Nghiệm thu

- [x] Adapter parse fixture list/insert/410.
- [x] Test qua `run()`: tạo sự kiện → reply sau khi fake trả OK, màn Lịch focus sự kiện mới.
- [x] Offline → hiển thị cache.

## Comments

Đã triển khai CalendarSource, Google adapter, cache SQLite và đồng bộ qua actor; test adapter, đồng bộ 410/offline và tạo sự kiện qua run. Chưa kiểm tra Google OAuth/API thực tế.

- Đồng bộ toàn bộ khi khởi động và mỗi ngày UTC để cửa sổ cache dịch chuyển vẫn nhận được sự kiện không thay đổi; test actor kiểm tra trường hợp này. OAuth đổi mã do Setup web thực hiện, adapter Google làm refresh. Chưa kiểm tra OAuth/API Google thực tế.

- Review: sửa tên `hub_calendar_get`, hỗ trợ sự kiện cả ngày và địa điểm; công bố trạng thái offline và giờ sync cuối lưu qua khởi động, mở màn khi tạo thất bại. Đã kiểm thử bằng fake và adapter HTTP cục bộ; chưa kiểm tra Google API thực tế.

- Review fix: đồng bộ đầy đủ gửi `timeMin`/`timeMax` (30 ngày trước – 60 ngày sau theo ngày địa phương) qua `CalendarQuery::Full`; đồng bộ đầy đủ hằng ngày tính theo ngày địa phương; lọc, sắp xếp và cắt cache so sánh theo thời điểm thay vì chuỗi.
