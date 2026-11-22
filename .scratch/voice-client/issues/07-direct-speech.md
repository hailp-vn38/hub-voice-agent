# 07 — Direct Speech

Status: ready-for-human
Blocked by: 05
Spec: [../spec.md](../spec.md)

## Việc

Theo ADR 0161 của vela-server (đang Proposed; kiểm trạng thái triển khai ở `.scratch/client-direct-speech/` trước khi làm).

- `DirectSpeech { request_id, text }` → `{"type":"direct_speech","session_id":…,"request_id":…,"text":…}`.
- Kiểm cục bộ trước khi gửi theo cùng luật server (`request_id` 1–128 ký tự `[A-Za-z0-9_.:-]`; `text` không rỗng, ≤ 1.024 ký tự và ≤ 4.096 byte, không ký tự điều khiển ngoài `\n` `\t`). Sai → `DirectSpeechRejected { reason: InvalidText | InvalidRequestId }`, không gửi.
- Chấp nhận: các frame `tts` mang `source: "client_direct"` → `TtsStart { origin: ClientDirect { request_id } }` (ticket 05). Frame `tts state:"result"` với outcome thành công không phát sự kiện riêng.
- Từ chối: `tts state:"result"`, `outcome:"rejected"`, `reason` → `DirectSpeechRejected { reason }`. `DirectSpeechError` có đủ mã server (`unsupported_feature`, `invalid_text`, `invalid_request_id`, `session_busy`, `rate_limited`, `tts_unavailable`, `capacity_reached`, `shutting_down`) cộng `Offline`, và một biến thể `Other(String)` cho mã lạ.
- Chưa Ready → `Offline` (bất biến 2).

## Nghiệm thu

- [x] Test: chấp nhận → `TtsStart { ClientDirect { request_id } }` … `TtsStop`; result thành công không phát event phụ.
- [x] Test: server trả `session_busy` → `DirectSpeechRejected { SessionBusy }`.
- [x] Test: text 1.025 ký tự → bị từ chối cục bộ, server giả không nhận gì.
- [x] Test: khi Offline → `DirectSpeechRejected { Offline }`.

## Comments

- `.scratch/client-direct-speech/spec.md` của repo vela-server vẫn ghi `needs-triage` và ADR 0161 vẫn `Proposed`. Trong worktree triển khai server, source hiện có parse `direct_speech`, ingress/admission vào session, validate, các mã lỗi, và TTS metadata `request_id`/`source`/`outcome`/`reason`. Triển khai client theo wire behavior trong source như ticket yêu cầu; các mã ngoài danh sách ticket được giữ nguyên trong `Other(String)`.
