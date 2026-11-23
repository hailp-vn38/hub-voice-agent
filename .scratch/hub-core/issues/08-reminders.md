# 08 — Reminder

Status: ready-for-agent
Blocked by: 06, 07
Spec: [../spec.md](../spec.md)

## Việc

- `src/features/reminders.rs`: `none` / `daily` / `weekly`, trạng thái đã nhắc / đã lỡ (lỡ < 1 giờ thì nhắc một lần), `create/cancel/snooze/due(now)`; `store/reminders.rs`.
- Đến hạn → `alerts.push(Reminder)` → `Play(ReminderChime)`; nếu online và setting đọc bằng giọng bật → `VoiceCommand::DirectSpeech{request_id}`. `DirectSpeechRejected(SessionBusy)` → thử lại sau; `Offline` → chỉ chuông + chữ. Alert tự tắt sau ~60 s, mục vào "Đã nhắc gần đây".
- Tool Reminder + `UiState.reminders`.

## Nghiệm thu

- [ ] Unit test `weekly` qua Chủ nhật, `daily` qua đổi tháng, lỡ 30 phút → nhắc, lỡ 2 giờ → chỉ đánh dấu đã lỡ.
- [ ] Test qua `run()`: đến hạn khi online → `DirectSpeech`; `SessionBusy` → gửi lại; Offline → không gửi.
- [ ] Huỷ theo nhãn dùng `selector`.
