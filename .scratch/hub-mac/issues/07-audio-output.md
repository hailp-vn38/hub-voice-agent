# 07 — Audio ra: Mixer, chuông, TTS

Status: ready-for-human
Asset attribution: needs-info
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

- [x] Test `Mixer`: `Play(Ding)` phát một lần rồi im; `Play(TimerChime)` lặp tới `StopSound(TimerChime)`; `StopTts` xoá hàng đợi TTS; chuông nhỏ hơn khi có TTS; thiếu dữ liệu → im lặng, không panic.
- [x] Test `sounds`: hai file giải mã được, ra mono đúng tần số yêu cầu.
- [ ] Tay: hẹn giờ 1 phút hết giờ → chuông lặp; bấm tắt → dừng ngay.
- [ ] Tay (vela-server): câu trả lời TTS phát ra tai nghe, không vấp.

## Comments

- A1 đầu ra đã nối `Mixer`/`sounds` qua audio thread và SPSC ring; Mac dùng thiết bị cpal mặc định, Pi chọn XVF3800. `Mic`, `SetVolume`, `SetWakeWord` được bỏ qua; không phát `AudioEvent` cho tới ticket 08.
- Attribution của hai MP3 vẫn `needs-info`: metadata chỉ cho biết codec/tần số/kênh, còn lịch sử git chỉ cho thấy chúng có sẵn trong commit nền `ba704ae`. Không sửa hai binary.
- Còn gate kiểm tay với thiết bị Mac và vela-server; chưa xác nhận runtime.
- 2026-11-23 review fix: `Mixer` giữ Ding một lần riêng với alert loop; `StopSound` dừng đúng alert và Ding không lặp. Regression test Mixer: 8 passed; test thiết bị: 3 passed; test voice bridge sau khi chuyển orchestration: 1 passed.
- Automated: `cargo check -p hub --no-default-features --features pi` pass; `scripts/test.sh -p hub` pass với 182 tests. Rustfmt cho các file Rust đã sửa và `git diff --check` pass.
- Kiểm tay trên Mac/phần cứng Pi và với vela-server vẫn pending; hai checkbox tay ở trên chưa được xác nhận.
