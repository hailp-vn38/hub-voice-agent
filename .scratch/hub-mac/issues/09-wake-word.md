# 09 — Wake word openWakeWord

Status: ready-for-agent
Blocked by: 08
Spec: [../spec.md](../spec.md) · ADR: [0005](../../../docs/adr/0005-local-wake-word-hardware-aec.md)

## Việc

- `wake.rs`: `OpenWakeWord` (melspectrogram → embedding → `hey_jarvis`) qua `ort`, cài `WakeDetector`. Ngưỡng độ nhạy vừa, là hằng số.
- Đọc model từ `AI_HUB_MODELS_DIR`, mặc định `models/` (docs §5). Thiếu file hoặc lỗi nạp → `Capture` chạy không có wake word, log một lần.
- Không phát hai `WakeWord` liền nhau trong ~2 s.
- Ghi cách tải ba file model vào [docs/development.md](../../../docs/development.md) "Âm thanh trên Mac" (model không commit).

## Nghiệm thu

- [ ] Test: thiếu model → `audio::spawn` vẫn thành công, không có `WakeWord`.
- [ ] Tay (tai nghe): "hey jarvis" → nghe tiếng ping, Orb sang nghe; nói tiếp → server trả lời.
- [ ] Tay: 10 phút nói chuyện bình thường gần máy → ghi số lần bắt nhầm vào Comments.
