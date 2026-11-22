# 08 — Đóng lượt TTS khi mất kết nối, cập nhật codebase.md

Status: ready-for-agent
Blocked by: 04, 05, 06, 07
Spec: [../spec.md](../spec.md)

## Việc

- Bất biến 1: socket đứt khi đã có `TtsStart` mà chưa `TtsStop` → phát `TtsStop { reason: Interrupted }` rồi mới `Connection(Offline)`. Server gửi `tts stop` sau `abort` → `Completed` như bình thường (server đã đóng lượt).
- Bất biến 4: test thứ tự sự kiện qua một kịch bản trộn `stt`, `tool`, `mcp tools/call`, `tts`.
- Rà lại toàn bộ bất biến 1–6 trong spec: mỗi cái có test chỉ đi qua interface công khai. Bổ sung test còn thiếu.
- Cập nhật [docs/codebase.md §3.1](../../../docs/codebase.md) theo bảng "Thay đổi so với codebase.md" trong spec, gồm phần "Giấu bên trong" và các bất biến. Sửa luồng ở §3.2/§4 nếu nhắc `Enrolled` hoặc `PlaybackAck`.

## Nghiệm thu

- [ ] Test: đứt giữa `TtsAudio` → `TtsStop { Interrupted }` đứng trước `Offline`.
- [ ] Test thứ tự trộn qua.
- [ ] Toàn bộ test của crate, clippy `-D warnings`, fmt qua `scripts/agent-test.sh`.
- [ ] `grep -n "Enrolled\|PlaybackAck" docs/` không còn kết quả.
