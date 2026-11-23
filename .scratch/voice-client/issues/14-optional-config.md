# 14 — `spawn` nhận `Option<VoiceConfig>`, channel sống suốt vòng đời app

Status: ready-for-agent
Blocked by: —
Spec: [../spec.md](../spec.md)

## Việc

Hiện `spawn(watch::Receiver<VoiceConfig>)` cần config ngay từ đầu, nên chưa có OTA URL thì `apps/hub` chưa gọi `spawn`. Lưu URL lần đầu, hoặc voice-client dừng, lại tạo ra một cặp channel mới. hub-core nhận `voice_tx`/`voice_rx` trong `Io` đúng một lần, nên giữa hai bên phải có một relay không test được (xem spec hub-core, thay đổi 1).

- Đổi thành `spawn(config: watch::Receiver<Option<VoiceConfig>>) -> (Sender<VoiceCommand>, Receiver<VoiceEvent>)`.
- `None` → không kết nối, phát `Connection(Unconfigured)`. Chuyển `Some → None` → đóng socket (lượt TTS đang mở thì phát `TtsStop { Interrupted }` trước), rồi phát `Unconfigured`. `None → Some` → kết nối như khi khởi động.
- Thêm `ConnectionState::Unconfigured`. Khi `Unconfigured`, command được xử lý như trước `Ready` (bất biến 2).
- Task client chỉ kết thúc khi receiver của config đóng; event channel đóng theo.

## Nghiệm thu

- [ ] Test: `spawn` với `None` → `Connection(Unconfigured)`, server giả không nhận kết nối nào; gửi `Some` → `Ready`.
- [ ] `Some → None` khi đang trong lượt TTS → `TtsStop { Interrupted }` rồi `Connection(Unconfigured)`.
- [ ] `DirectSpeech` khi `Unconfigured` → `DirectSpeechRejected { Offline }`.
- [ ] Bất biến 6 vẫn giữ: gửi lại cùng `Some(config)` không kết nối lại.
- [ ] `docs/codebase.md` §3.1 cập nhật chữ ký và bất biến.
