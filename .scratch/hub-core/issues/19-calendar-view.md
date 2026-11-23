# 19 — Lịch: dữ liệu hiển thị

Status: ready-for-agent
Blocked by: 14, 17
Spec: [../spec.md](../spec.md) · UI: hub-ui #10

## Việc

- `present`: `calendar` — `month-title`, `synced` + `stale` (từ `calendar_online`, `calendar_last_sync`), `week` (7 ngày, `dots`, `today`), `blocks` của ngày chọn gồm sự kiện lịch **và** Reminder (`kind`), `start-min/end-min`, `all-day`, `ongoing`, `col/cols` khi trùng giờ, `now-min`, `upcoming` (gộp `more`), `month` (lưới tháng), `detail`, `focus` + serial.
- Chuyển thuật toán xếp cột từ `apps/hub/src/ui/demo/fixtures/calendar/columns.rs` (nhánh `feat/hub-ui`) vào `present/calendar`, cùng test của nó; fixture demo gọi lại hàm này hoặc giữ bản sao tới khi gộp nhánh.
- `selected-day` do hub-core giữ (#14): suy từ `Focus::Calendar` (id sự kiện → ngày bắt đầu của sự kiện; ngày `YYYY-MM-DD` → ngày đó; không có focus → hôm nay). Chạm `select-day` → `UserInput::Focus(Some(Focus::Calendar(ngày)))`. `month-view` và `detail` do UI giữ; `present` cấp sẵn `month` và chi tiết của mọi sự kiện trong ngày chọn để UI mở mà không cần hỏi lại.
- `PendingState` (saving / saved / failed) cho sự kiện vừa tạo bằng `hub_calendar_create`, theo trạng thái job.

## Nghiệm thu

- [ ] Test `present`: 3 sự kiện chồng giờ → `cols == 3`, `col` 0..2; sự kiện cả ngày không vào dòng thời gian.
- [ ] Test: focus id sự kiện thứ Sáu → `selected-day` là thứ Sáu, `blocks` là của thứ Sáu.
- [ ] Test: Reminder hôm nay hiện trong `blocks` với `kind == reminder`.
- [ ] Test qua `run()`: tạo sự kiện → `saving` rồi `saved`; nguồn lỗi → `failed`.

## Comments

- 2026-11-23: đã chốt ở #14. `selected-day` do hub-core giữ; `month-view`, `detail` do UI giữ. Vì vậy hợp đồng `CalDetail` cần chuyển thành danh sách chi tiết theo `key` (UI chọn theo key đang mở): ghi vào ticket hub-ui #10 khi map.
