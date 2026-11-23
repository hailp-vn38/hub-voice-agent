# 10 — Màn Lịch

Status: resolved
Blocked by: 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Lịch"; ví dụ "tạo sự kiện" ở "Agent điều khiển hub"

## Việc

- `screens/calendar.slint` + `components/calendar/` (dải 7 ngày, dòng thời gian, lưới tháng, thẻ chi tiết).
- **Đầu màn:** "‹ Lịch · Tháng {N}" (chạm tên tháng → lưới tháng); "Đồng bộ HH:MM" bên phải, badge offline khi cũ.
- **Dải 7 ngày:** hôm nay → 6 ngày tới, ô lớn; ngày chọn tô màu nhấn; tối đa 3 chấm theo số sự kiện.
- **Trái — dòng thời gian ngày chọn:** dải "Cả ngày" trên cùng; khối sự kiện theo giờ; **vạch "bây giờ"** đỏ nhấp nháy nhẹ; sự kiện đang diễn ra sáng hơn; Reminder là khối **viền đứt nét**; cuộn dọc được.
- **Phải — Sắp tới:** 5 sự kiện kế tiếp sau ngày chọn; ngày nhiều sự kiện gộp "+N sự kiện".
- **Lưới tháng** thay dòng thời gian + Sắp tới; chấm sự kiện mỗi ngày; chạm ngày → về dòng thời gian ngày đó.
- **Chạm sự kiện** → thẻ chi tiết: tiêu đề, giờ, địa điểm, mô tả.
- **Sự kiện mới tạo bằng giọng:** dùng `SpawnIn` (ticket 06) — `saving` → `saved` bay vào đúng ngày trong dải và "Sắp tới", nền xanh nhạt, ghi "Mới tạo"; `failed` rung, viền đỏ.
- Không cuộn ngang.
- `contract/calendar.slint`:
  - `CalDay { key, weekday, day, dots: int, today: bool }`, `CalBlock { key, kind: CalKind, title, time, start-min: int, end-min: int, all-day: bool, ongoing: bool, fresh: bool, pending: PendingState }` với `export enum CalKind { event, reminder }`, `CalUpcoming { key, date, time, title, more: int }`, `CalMonthDay { key, day, dots: int, in-month: bool, today: bool }`, `CalDetail { visible: bool, title, time, location, description }`.
  - `global CalendarData { in property <string> month-title; in property <string> synced; in property <bool> stale; in property <[CalDay]> week; in property <string> selected-day; in property <[CalBlock]> blocks; in property <int> now-min; in property <[CalUpcoming]> upcoming; in property <bool> month-view; in property <[CalMonthDay]> month; in property <CalDetail> detail; in property <string> focus; in property <int> focus-serial; callback select-day(string); callback toggle-month(); callback open-detail(string); callback close-detail(); }`.
  - `start-min`/`end-min` là phút trong ngày để Slint đặt khối; Slint không tính ngày.

## Demo

Fixture: tuần có sự kiện cả ngày, sự kiện trùng giờ, Reminder. Kịch bản: "Hôm nay có gì?" (cuộn tới sự kiện kế tiếp), "Thứ Sáu có gì?", "Lịch tiếp theo" (nhấp sáng ở cả hai cột), "Tháng sau có gì?" (lưới tháng), "Tạo khám răng thứ Ba → thành công", "Tạo → lỗi", "Offline".

## Nghiệm thu

- [ ] Vạch "bây giờ" đúng vị trí theo `now-min`, dời mỗi phút.
- [ ] Reminder phân biệt rõ với sự kiện Google (viền đứt nét).
- [ ] Sự kiện trùng giờ không đè chữ lên nhau.
- [ ] Sự kiện mới đi đúng thứ tự Orb → thẻ "đang lưu" → bay vào ngày.

## Map với hub-core

Map: chờ hub-core #10 (lịch), #08 (Reminder trên dòng thời gian)

| Hợp đồng | UiState / UserInput |
|---|---|
| `CalendarData.month-title` / `.synced` / `.stale` | |
| `.week` / `.selected-day` | |
| `.blocks` / `.now-min` | |
| `.upcoming` | |
| `.month-view` / `.month` | |
| `.detail` | |
| `.focus` / `.focus-serial` | |
| `select-day` / `toggle-month` / `open-detail` / `close-detail` | |

## Answer

Done: contract/calendar.slint, screens/calendar.slint, components/calendar/*, demo fixtures + 7 scenarios (fixtures/calendar/). Deviations: CalBlock gained col/cols (adapter assigns columns for overlaps, unit-tested) and CalUpcoming gained fresh; new event flies into the selected day timeline (and Upcoming for later days), week strip only gains a dot; "Lich tiep theo" lights the next event in whichever column holds it. Reminder dashes drawn as segments. Not visually verified.
