# 03 — Conversation + kênh audio

Status: ready-for-human
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- `src/audio.rs`: `AudioCommand`, `AudioEvent`, `Sound` như codebase.md §3.2. `Io` thêm `audio_tx`, `audio_rx`.
- `src/conversation.rs`: hàm thuần `on(input, now) -> Vec<Effect>`, trạng thái Idle / Listening / Thinking / Speaking / Offline; `turn_active()`; về Idle sau 10 s im lặng kể từ `TtsStop`.
- `runtime/voice.rs`, `runtime/audio.rs`: nối luồng wake word → `ListenStart` + `Play(Ding)` + `Mic{on:true}`; `MicFrame` → `VoiceCommand::Audio` chỉ khi đang nghe; `TtsAudio` → `PlayTts`; `TtsStop{Interrupted}` → `StopTts`.
- `UiState.voice { state, stt_text, response_text }`.
- `apps/hub`: adapter audio giả trên desktop (chưa có `cpal`) để build được.

## Nghiệm thu

- [x] Unit test `conversation` cho mọi chuyển trạng thái, gồm Offline khi `Connection(Offline)` và wake word khi Offline (không gửi `ListenStart`).
- [x] Test qua `run()`: WakeWord → đúng 3 command; tiến clock 10 s sau `TtsStop` → `ListenStop` + `Mic{on:false}`.
- [x] `MicFrame` khi Idle bị bỏ.

## Comments

- Đã kiểm thử state machine và luồng qua `run()` với clock ảo; adapter desktop là giả, chưa kiểm thử thu/phát audio hoặc thiết bị thật.
