# 10 — Mức âm cho Orb

Status: ready-for-human
Blocked by: 01, 08
Spec: [../spec.md](../spec.md) · UI: hub-ui #04

## Việc

- `Levels { mic, tts }` (0..1): `Capture::level()` và `Mixer::level()`. Callback audio chỉ chuyển PCM qua ring buffer; thread audio đọc mức và `send_replace` vào `watch` mỗi ~33 ms.
- `bridge::spawn` nhận `levels_rx`, ghi `VoiceLevels.mic`/`tts` qua `invoke_from_event_loop`.
- Điền dòng `VoiceLevels` trong bảng Map của hub-ui #04.

## Nghiệm thu

- [ ] Tay: nói trong lượt → Orb nhịp theo giọng; TTS phát → Orb nhịp theo TTS; im lặng → Orb đứng yên.
- [ ] Tay: CPU của app khi rảnh không tăng rõ so với trước ticket (ghi số vào Comments).

## Comments

- 2026-11-23: Đã merge integration `feat/hub-mac` tại `ad4a918` vào branch ticket 10 (`82f55ab`).
- Automated: `scripts/test.sh -p hub` pass, `ok: 179 passed`.
- `rustfmt` có mục tiêu pass cho `main.rs`, `audio/mod.rs`, `audio/capture_tests.rs`, `audio/mixer_tests.rs`, `ui/bridge/voice.rs`, và `ui/bridge/voice_tests.rs`.
- `git diff --check ad4a918..82f55ab` và `git diff --cached --check` cho tracker update đều pass.
- Còn chờ người kiểm tra Orb trên Mac với mic/TTS/im lặng và đo CPU lúc rảnh; hai mục nghiệm thu thủ công vẫn pending.
