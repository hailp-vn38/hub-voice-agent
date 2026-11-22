# 05 — Listen, audio uplink, STT/TTS downlink

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- `ListenStart` → `{"type":"listen","state":"start","mode":"realtime"}`; `ListenStop` → `state: stop`; `Abort` → `{"type":"abort"}`. Kiểm schema với `protocol/` của server.
- `Audio(OpusFrame)` → binary frame raw Opus, không header.
- Downlink: `stt` → `Stt`; `tts start` → `TtsStart { origin }` (`source: "client_direct"` + `request_id` → `ClientDirect`, còn lại `Agent`); `sentence_start` → `TtsText`; binary → `TtsAudio`; `tts stop` → `TtsStop { reason: Completed }`.
- Message lạ hoặc sai schema: log rồi bỏ qua, không đứt kết nối (giống Protocol Fault Boundary của server).
- Bất biến 2 (lệnh khi chưa Ready) và bất biến 5 (capacity, drop `TtsAudio` khi channel sự kiện đầy).

## Nghiệm thu

- [ ] Test: một lượt đầy đủ `listen start` + 3 frame audio → server giả nhận đúng thứ tự → gửi `stt`, `tts start`, `sentence_start`, 2 Opus, `tts stop` → nhận đúng chuỗi `VoiceEvent`.
- [ ] Test: `Audio` trước `Ready` không tới server.
- [ ] Test: frame JSON lạ giữa lượt → lượt vẫn tiếp tục.
- [ ] Test: không đọc `VoiceEvent` trong khi server đẩy nhiều Opus → reader WS không bị chặn, `TtsStop` vẫn tới khi đọc lại.
