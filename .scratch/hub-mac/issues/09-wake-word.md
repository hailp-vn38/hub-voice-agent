# 09 — Wake word openWakeWord

Status: ready-for-human
Blocked by: 08
Spec: [../spec.md](../spec.md) · ADR: [0005](../../../docs/adr/0005-local-wake-word-hardware-aec.md)

## Việc

- `wake.rs`: `OpenWakeWord` (melspectrogram → embedding → `hey_jarvis`) qua `ort`, cài `WakeDetector`. Ngưỡng độ nhạy vừa, là hằng số.
- Đọc model từ `AI_HUB_MODELS_DIR`, mặc định `models/` (docs §5). Thiếu file hoặc lỗi nạp → `Capture` chạy không có wake word, log một lần.
- Không phát hai `WakeWord` liền nhau trong ~2 s.
- Ghi cách tải ba file model vào [docs/development.md](../../../docs/development.md) "Âm thanh trên Mac" (model không commit).

## Nghiệm thu

- [x] Test ở seam Capture: thiếu model → detector được bỏ qua và Capture không phát `WakeWord`.
- [ ] Tay (tai nghe): "hey jarvis" → nghe tiếng ping, Orb sang nghe; nói tiếp → server trả lời.
- [ ] Tay: 10 phút nói chuyện bình thường gần máy → ghi số lần bắt nhầm vào Comments.

## Comments

- 2026-11-23: Đã thêm pipeline openWakeWord ba model ONNX, threshold cố định 0.5, debounce 2 giây và fallback khi thiếu/lỗi model. `AI_HUB_MODELS_DIR` ưu tiên hơn thư mục mặc định `models/`.
- Automated: `scripts/test.sh -p hub` lần đầu có 103 pass và một timeout ngoài hub ở `hub-core/tests/timers.rs::timers_are_saved_reloaded_and_limit_is_reported`; lần chạy lại duy nhất pass với 176 test. Rustfmt cho các file audio đã sửa và `git diff --check` pass.
- Test thiếu model không mở `cpal`; nó kiểm tra detector tùy chọn và Capture không tạo `WakeWord`. Việc gọi `audio::spawn` cùng thiết bị Mac, nói wake word thật, và đếm bắt nhầm trong 10 phút vẫn cần kiểm tra thủ công.
