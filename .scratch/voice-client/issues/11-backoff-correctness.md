# 11 — Backoff: trần sau jitter, reset chỉ khi kết nối ổn định

Status: done
Spec: [../spec.md](../spec.md)
Nguồn: review nhánh feat/voice-client (2026-11-23)

## Việc

- `lib.rs`: áp jitter rồi mới chặn trần 60 s (spec: "tối đa 60 s").
- Chỉ reset backoff về 1 s khi kết nối Ready đã sống ≥ ngưỡng (vd. 30 s); đóng ngay sau hello thì tiếp tục tăng.
- Đặt tên hằng cho ngưỡng; cập nhật spec nếu thêm ngưỡng ổn định.

## Nghiệm thu

- [x] Test `retry_in` không bao giờ > 60 s.
- [x] Test server hello rồi đóng ngay lặp lại → backoff tăng dần.

## Comments

- 2026-11-23: `retry_in = jitter(backoff).min(MAX_BACKOFF)`. Ngưỡng ổn định `Timeouts::stable_after` (production 30 s, `NONE` = 0 để test cũ giữ hành vi reset ngay). `ConnectionEnd::Failed { became_ready }` đổi thành `was_stable`, do `connect_and_wait` tính từ lúc Ready. Spec mục Backoff đã cập nhật.
