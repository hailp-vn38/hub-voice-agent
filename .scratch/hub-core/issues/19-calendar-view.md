# 19 — Lịch: dữ liệu hiển thị

Status: completed
Blocked by: 14, 17
Spec: [../spec.md](../spec.md) · UI: hub-ui #10

## Việc

- `present`: `calendar` — `month-title`, `synced` + `stale` (từ `calendar_online`, `calendar_last_sync`), `week` (7 ngày, `dots`, `today`), `blocks` của ngày chọn gồm sự kiện lịch **và** Reminder (`kind`), `start-min/end-min`, `all-day`, `ongoing`, `col/cols` khi trùng giờ, `now-min`, `upcoming` (gộp `more`), `month` (lưới tháng), `detail`, `focus` + serial.
- Chuyển thuật toán xếp cột từ `apps/hub/src/ui/demo/fixtures/calendar/columns.rs` (nhánh `feat/hub-ui`) vào `present/calendar`, cùng test của nó; fixture demo gọi lại hàm này hoặc giữ bản sao tới khi gộp nhánh.
- `selected-day` do hub-core giữ (#14): suy từ `Focus::Calendar` (id sự kiện → ngày bắt đầu của sự kiện; ngày `YYYY-MM-DD` → ngày đó; không có focus → hôm nay). Chạm `select-day` → `UserInput::Focus(Some(Focus::Calendar(ngày)))`. `month-view` và `detail` do UI giữ; `present` cấp sẵn `month` và chi tiết của mọi sự kiện trong ngày chọn để UI mở mà không cần hỏi lại.
- `PendingState` (saving / saved / failed) cho sự kiện vừa tạo bằng `hub_calendar_create`, theo trạng thái job.

## Nghiệm thu

- [x] Test `present`: 3 sự kiện chồng giờ → `cols == 3`, `col` 0..2; sự kiện cả ngày không vào dòng thời gian.
- [x] Test: focus id sự kiện thứ Sáu → `selected-day` là thứ Sáu, `blocks` là của thứ Sáu.
- [x] Test: Reminder hôm nay hiện trong `blocks` với `kind == reminder`.
- [x] Test qua `run()`: tạo sự kiện → `saving` rồi `saved`; nguồn lỗi → `failed`.

## Comments

- 2026-11-23: đã chốt ở #14. `selected-day` do hub-core giữ; `month-view`, `detail` do UI giữ. Vì vậy hợp đồng `CalDetail` cần chuyển thành danh sách chi tiết theo `key` (UI chọn theo key đang mở): ghi vào ticket hub-ui #10 khi map.

- 2026-11-23: Đã thêm `CalendarView` cho tuần, lưới tháng, block sự kiện/Reminder, cột chồng giờ, sắp tới, chi tiết theo `key`, trạng thái đồng bộ và focus. `hub_calendar_create` chọn ngày sự kiện ngay khi bắt đầu; trạng thái hiển thị `saving` → `saved`/`failed`, kể cả timeout. Google `CalendarEvent.description` được đưa vào chi tiết. `tests/present_calendar.rs`, `tests/calendar_focus.rs`, `tests/calendar_create.rs` và `tests/job_timeout.rs` kiểm tra qua public `present()`/`run()`.
- 2026-11-23: Gate tự động: `CARGO_TARGET_DIR=/private/tmp/ai-hub-core-19/target scripts/test.sh -p hub-core` (134 passed), `cargo fmt --all -- --check`, `cargo build -p hub`, `git diff --check` đều qua. Map và kiểm tra trực quan thuộc hub-ui #10, chưa chạy; khi map cần đổi `CalDetail` thành danh sách có `key`.
- 2026-11-23: Hoàn tất các tiêu chí hub-core. Gate cuối sau tích hợp tickets 14–21: `scripts/test.sh -p hub-core -- --test-threads=1` (143 passed), `cargo fmt --all -- --check`, `cargo build -p hub`, `git diff --check 86fafda..HEAD` đều qua. Map và kiểm tra trực quan tiếp tục thuộc hub-ui #10; khi map cần đổi `CalDetail` thành danh sách có `key`.
- 2026-11-23 (hub-mac #04, commit `1a6cd0c`): UI bridge giữ `month-view` và key chi tiết, lấy `.details` từ `HubView.calendar` để dựng `CalDetail`; với mục Sắp tới dùng tạm `.upcoming`. Không cần đổi hợp đồng `CalDetail` hoặc hub-core.
