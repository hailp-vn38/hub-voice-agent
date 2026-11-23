# 10 — Lịch Google

Status: ready-for-agent
Blocked by: 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `CalendarSource`; adapter `sources/google_calendar.rs` (events.list với `syncToken`, events.insert, OAuth đổi mã + refresh theo ADR 0007). Refresh token lưu trong `ConfigStore`.
- `src/features/calendar/` (khuôn ticket 04): cache ~60 ngày, `apply(delta)`, `events(range)`, ghi nhớ sự kiện mới tạo.
- Làm mới 5 phút trên tick của actor qua `Effect::Spawn`; tạo sự kiện cũng qua `Effect::Spawn` kèm `ToolCall`; `syncToken` hết hạn (410) → đồng bộ lại toàn bộ.
- Tool xem/tạo sự kiện.

## Nghiệm thu

- [ ] Adapter parse fixture list/insert/410.
- [ ] Test qua `run()`: tạo sự kiện → reply sau khi fake trả OK, màn Lịch focus sự kiện mới.
- [ ] Offline → hiển thị cache.
