# 13 — Màn Reminder

Status: ready-for-agent
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
| `RemindersData.title` | |
| `.has-next` / `.next` | |
| `.groups` | |
| `.recent` | |
| `.focus` / `.focus-serial` | |
