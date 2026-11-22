# 04 — Reconnect có backoff, đổi config

Status: ready-for-agent
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

- Mọi lần đứt (OTA lỗi, WS lỗi, server đóng, write timeout) → `Connection(Offline { retry_in })` rồi thử lại sau `retry_in`.
- Backoff: 1 s nhân đôi tới 60 s, jitter ±20 %, reset sau khi `Ready`. Jitter lấy từ nguồn ngẫu nhiên inject được để test xác định.
- Đóng `1000` sau Enrollment: kết nối lại ngay (ticket 03).
- `watch` của `VoiceConfig` đổi sang giá trị khác → đóng kết nối hiện tại (đóng `1000`) rồi kết nối lại ngay với config mới. Giá trị bằng nhau thì bỏ qua.
- Sender `VoiceCommand` bị drop hoặc receiver `VoiceEvent` bị drop → đóng socket, task kết thúc.

## Nghiệm thu

- [ ] Test (`tokio::time::pause`): server từ chối liên tiếp → `retry_in` tăng 1, 2, 4… tới trần 60 s trong biên jitter.
- [ ] Test: sau một lần `Ready` rồi đứt → `retry_in` quay về ~1 s.
- [ ] Test: đổi `ota_url` → kết nối tới server giả thứ hai.
- [ ] Test: gửi lại cùng giá trị config → không kết nối lại.
- [ ] Test: drop cả hai đầu channel → task kết thúc, server giả thấy close.
