---
status: accepted
---

# VAD ở server, chế độ realtime, không gửi listen:detect

Hub không chạy VAD; sau wake word hoặc chạm Orb nó gửi `listen:start` ở chế độ `realtime` và stream Opus 60 ms liên tục, server cắt câu và xử lý barge-in. Hub **không** gửi `listen:detect` khi nghe wake word, vì server coi `text` của `listen:detect` là một lượt nói đã gõ và chạy LLM ngay ("hey jarvis" sẽ thành câu hỏi thật).

Khi Idle, WebSocket vẫn giữ ở Ready (kết nối, không gửi mic) để tránh trễ admission mỗi lần đánh thức. Hội thoại kết thúc khi 10 giây không có lời nói kể từ lúc TTS dừng: hub ngừng gửi mic và về Idle.

## Consequences

- ClientHello khai báo `features.aec=true`; server phải bật `barge_in.enabled` và `barge_in.trust_client_aec_feature`.
- Cần ring buffer ngắn sau wake word để không mất audio trong lúc gửi `listen:start`.
