# 13 — Màn Reminder

Status: ready-for-human
Blocked by: 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Reminder" (trạng thái nhắc toàn màn ở ticket 14)

## Việc

- `screens/reminders.slint`.
- **Đầu màn:** "‹ Reminder · N sắp tới".
- **Trái — Kế tiếp:** giờ lớn, "còn X phút/giờ", nội dung, "lặp hằng ngày" nếu có.
- **Phải — dòng thời gian:** nhóm theo ngày (Hôm nay / Ngày mai / …); mỗi dòng: giờ, nội dung, icon lặp.
- **Dưới — Đã nhắc gần đây:** 3–5 mục trong 24 giờ; mục lỡ ghi "Đã lỡ".
- **Trống:** "Hey Jarvis, nhắc tôi uống thuốc lúc 9 giờ tối".
- **Hiệu ứng:** mục mới `SpawnIn` từ Orb vào dòng thời gian; huỷ → mục mờ dần; hoãn → mục dời giờ trên dòng thời gian; focus giọng nói → các mục `focus` sáng lên.
- `contract/reminders.slint`:
  - `ReminderItem { key, time, text, repeat-text, repeats: bool }`, `ReminderGroup { label, items: [ReminderItem] }`, `ReminderNext { key, time, until, text, repeat-text }`, `ReminderPast { key, time, text, missed: bool }`.
  - `global RemindersData { in property <string> title; in property <bool> has-next; in property <ReminderNext> next; in property <[ReminderGroup]> groups; in property <[ReminderPast]> recent; in property <[string]> focus; in property <int> focus-serial; }`.

## Demo

Kịch bản: "Nhắc uống thuốc 21:00" (spawn), "Mỗi sáng 7 giờ tập thể dục" (icon lặp), "Tối nay có nhắc gì?" (focus nhiều mục), "Huỷ nhắc uống thuốc", "Hoãn 10 phút", "Một mục đã lỡ".

## Nghiệm thu

- [ ] Ba vùng đúng như concept; nhóm theo ngày có tiêu đề rõ.
- [ ] "Đã lỡ" phân biệt rõ với mục đã nhắc.
- [ ] Focus nhiều mục cùng lúc sáng đúng các mục.

## Map với hub-core

Map: chờ hub-core #08

| Hợp đồng | UiState / UserInput |
|---|---|
| `RemindersData.title` | `HubView.reminders.title` |
| `.has-next` / `.next` | `HubView.reminders.has_next`, `.next` |
| `.groups` | `HubView.reminders.groups`, gồm key, giờ, nội dung, chu kỳ và cờ lặp của từng mục |
| `.recent` | `HubView.reminders.recent` |
| `.focus` / `.focus-serial` | `RemindersView` chưa có; bridge giữ mặc định Slint |
| `ReminderItem.fading` | `ReminderItemView` chưa có; bridge đặt `false` |

Map: done

## Answer

Đã làm `screens/reminders.slint`, `contract/reminders.slint` (thêm `fading: bool` vào `ReminderItem` để huỷ mờ dần), fixture và 7 kịch bản trong `demo/reminders.rs`. Mục mới dùng `SpawnIn`; focus viền sáng theo `RemindersData.focus` (không dùng `focus-serial`). Chưa có test Rust (không có logic thuần). Map hub-core chờ #08.

## Comments

- 2026-11-23 (review hub-ui): Focus giờ theo `focus-serial`: các mục trong `focus` sáng lên ~2 s rồi tắt dần. Hoãn: dòng trượt sang giờ mới (giờ đổi, cùng key); demo sửa mục tại chỗ thay vì dựng lại model. Huỷ cũng sửa tại chỗ để thấy mờ dần. Ghi `ReminderItem.fading` vào bảng Map. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
