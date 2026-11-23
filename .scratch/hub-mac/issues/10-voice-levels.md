# 10 — Mức âm cho Orb

Status: ready-for-agent
Blocked by: 01, 08
Spec: [../spec.md](../spec.md) · UI: hub-ui #04

## Việc

- `Levels { mic, tts }` (0..1): `Capture::level()` và `Mixer::level()` (callback ghi qua atomic). Thread audio `send_replace` vào `watch` mỗi ~33 ms.
- `bridge::spawn` nhận `levels_rx`, ghi `VoiceLevels.mic`/`tts` qua `invoke_from_event_loop`.
- Điền dòng `VoiceLevels` trong bảng Map của hub-ui #04.

## Nghiệm thu

- [ ] Tay: nói trong lượt → Orb nhịp theo giọng; TTS phát → Orb nhịp theo TTS; im lặng → Orb đứng yên.
- [ ] Tay: CPU của app khi rảnh không tăng rõ so với trước ticket (ghi số vào Comments).
