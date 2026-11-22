---
status: accepted
---

# Hẹn giờ, Đếm ngược, Reminder chạy và lưu trên hub

Hẹn giờ, Đếm ngược và Reminder đều chạy và lưu cục bộ trên hub, với thời điểm lưu theo giờ tuyệt đối, để còn hoạt động khi mất mạng và sau khi khởi động lại. Hub **không** dùng Reminder của vela-server và không dùng Device Push Channel (MQTT): Push Channel chỉ mang Wake Signal cho Reminder phía server, còn hub tự có đủ nội dung.

Nội dung Reminder được đọc bằng giọng server qua message mới `direct_speech` (hub gửi văn bản, server TTS thẳng, không qua ASR/LLM, không vào lịch sử hội thoại; ClientHello bật `features.direct_tts=true`). Thiết kế phía server: vela-server ADR 0161, đang Proposed.

## Consequences

- Agent của hub phải dùng `hub_reminder_*`; chặn tool Reminder builtin của server cho Device này (allowlist hoặc prompt), nếu không agent tạo Reminder mà hub không thấy.
- Mất mạng hoặc server từ chối (`session_busy`, `rate_limited`): chỉ chuông + hiện chữ; bận thì đợi rồi gửi lại.
- Mốc `yearly_lunar` cần bộ chuyển đổi âm–dương lịch Việt Nam chạy trên hub.
