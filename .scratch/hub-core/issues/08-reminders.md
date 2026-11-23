# 08 — Reminder

Status: completed
Blocked by: 06, 07
Spec: [../spec.md](../spec.md)

## Việc

- `src/features/reminders/` (khuôn ticket 04): `none` / `daily` / `weekly`, trạng thái đã nhắc / đã lỡ (lỡ < 1 giờ thì nhắc một lần), `create/cancel/snooze/due(now)`; `features/reminders/store.rs`.
- Đến hạn → `alerts.push(Reminder)` → `Play(ReminderChime)`; nếu online và setting đọc bằng giọng bật → `VoiceCommand::DirectSpeech{request_id}`. `DirectSpeechRejected(SessionBusy)` → thử lại sau; `Offline` → chỉ chuông + chữ. Alert tự tắt sau ~60 s, mục vào "Đã nhắc gần đây".
- Tool Reminder + `UiState.reminders`.

## Nghiệm thu

- [x] Unit test `weekly` qua Chủ nhật, `daily` qua đổi tháng, lỡ 30 phút → nhắc, lỡ 2 giờ → chỉ đánh dấu đã lỡ.
- [x] Test qua `run()`: đến hạn khi online → `DirectSpeech`; `SessionBusy` → gửi lại; Offline → không gửi.
- [x] Huỷ theo nhãn dùng `selector`.

## Comments

- Đã triển khai Reminder, thông báo và công cụ; test qua `run()` bao phủ online, SessionBusy, offline và nhắc lỡ. Test qua `run()` xác nhận huỷ theo nhãn không dấu và snooze giữ lịch lặp.

- Review fix: `at` nhận giờ có múi (`…[Asia/Ho_Chi_Minh]`), RFC 3339 hoặc giờ địa phương không offset; lưu dạng có múi. Reminder lặp bị lỡ nhiều ngày ghi một mục "đã lỡ" rồi nhảy thẳng tới lần kế tiếp chưa lỡ.
