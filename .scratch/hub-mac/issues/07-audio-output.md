# 07 — Audio ra: Mixer, chuông, TTS

Status: ready-for-agent
Blocked by: —
Spec: [../spec.md](../spec.md) · Thiết kế: [../design.md](../design.md) §2

## Việc

- `src/audio/{mod.rs, devices.rs, mixer.rs, sounds.rs}`; `audio::spawn` dựng thread audio, mở stream ra trên chính thread đó (`cpal::Stream` không `Send` trên macOS).
- `Mixer` thuần: `apply(AudioCommand)`, `pull(&mut [f32])`, `level()`.
  - `PlayTts`: decode Opus 24 kHz (`opus` crate) ở thread audio, resample lên tần số thiết bị (`rubato`), đưa vào hàng đợi TTS. `StopTts` xoá hàng đợi.
  - `Play`/`StopSound` theo bảng ở [../spec.md](../spec.md) quyết định 6; chuông nhỏ hơn khi đang có TTS.
  - `Mic`, `SetVolume`, `SetWakeWord` bỏ qua ở ticket này.
- `sounds.rs`: `include_bytes!` hai file trong `assets/audio/`, giải mã bằng `symphonia` một lần lúc `spawn`, đổi về mono và tần số thiết bị.
- Callback cpal không cấp phát, không khoá: lệnh và PCM đi qua ring SPSC (`rtrb`).
- `main.rs`: dùng `AudioHandle.commands`/`events` cho `Io`, bỏ vòng drain. `events` chưa phát gì cho tới #08.
- Ghi nguồn và giấy phép của `ping.mp3`, `reminder.mp3` vào `apps/hub/assets/README.md`. Chưa rõ nguồn → để `needs-info` phần này, không commit hai file.
- Ghi `brew install opus` vào [docs/development.md](../../../docs/development.md) "Âm thanh trên Mac".

## Nghiệm thu

- [ ] Test `Mixer`: `Play(Ding)` phát một lần rồi im; `Play(TimerChime)` lặp tới `StopSound(TimerChime)`; `StopTts` xoá hàng đợi TTS; chuông nhỏ hơn khi có TTS; thiếu dữ liệu → im lặng, không panic.
- [ ] Test `sounds`: hai file giải mã được, ra mono đúng tần số yêu cầu.
- [ ] Tay: hẹn giờ 1 phút hết giờ → chuông lặp; bấm tắt → dừng ngay.
- [ ] Tay (vela-server): câu trả lời TTS phát ra tai nghe, không vấp.
