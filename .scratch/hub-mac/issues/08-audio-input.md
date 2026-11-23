# 08 — Audio vào: Capture, gửi tiếng mic

Status: ready-for-human
Blocked by: 01, 07
Spec: [../spec.md](../spec.md) · Thiết kế: [../design.md](../design.md) §2.2, §2.3

## Việc

- `src/audio/capture.rs` (+ `wake.rs` chỉ có trait): `Capture::new(wake: Option<Box<dyn WakeDetector>>)`, `apply(&AudioCommand)` (`Mic { on }`), `push(&[i16]) -> Vec<AudioEvent>`, `level()`.
  - Khối 80 ms cho wake word (nếu có), preroll ~1,5 s, encode Opus 16 kHz mono khối 60 ms.
  - `Mic { on: true }` → xả preroll thành `MicFrame` trước, rồi gửi frame mới; mic tắt → không gửi `MicFrame`.
  - Wake word phát `AudioEvent::WakeWord` (khi có detector).
- `devices.rs`: mở stream vào trên thread audio, xin 16 kHz mono; không được thì resample từ tần số gốc. Mẫu qua ring SPSC.
- Thread audio gửi `AudioEvent` qua `blocking_send`.

## Nghiệm thu

- [x] Test `Capture` với detector giả: không có `MicFrame` khi mic tắt; sau `WakeWord` và `Mic { on: true }` frame đầu là preroll; frame là Opus giải mã được, 60 ms ở 16 kHz.
- [x] Test resample: 48 kHz vào → khối 960 mẫu ra đúng nhịp 60 ms.
- [ ] Tay (tai nghe, vela-server): chạm Orb, nói "mấy giờ rồi" → STT hiện chữ, server trả lời bằng TTS.
- [ ] Tay: hỏi bằng giọng một tool `hub_*` (vd. hẹn giờ) → mở đúng màn.

## Comments

- 2026-11-23: Sau khi merge `feat/hub-mac` tại `147db392`, `scripts/test.sh -p hub` đạt `ok: 173 passed`. Kiểm tra tai nghe/vela-server và luồng tool bằng giọng nói trên Mac còn chờ người kiểm tay.
