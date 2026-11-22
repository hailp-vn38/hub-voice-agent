# 03 — Enrollment Session và mã kích hoạt

Status: ready-for-agent
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

Theo ADR 0074 của vela-server: Device chưa đăng ký vẫn được upgrade nhưng vào Enrollment Session. Server gửi hello, `stt` chứa mã, rồi `tts` + Opus đọc mã. Khi được claim, server gửi thông báo rồi đóng `1000`. Client phải tự kết nối lại để vào Voice Session thật.

- **Trước khi code:** tìm trong `crates/vela-server/src/app/enrollment/` và transport WS cách client phân biệt Enrollment Session với Voice Session (field trong ServerHello, dạng `stt`, hay chỉ suy từ OTA). Ghi kết luận vào `## Comments`. Nếu không có tín hiệu rõ ràng, chuyển ticket sang `needs-info` và mô tả lựa chọn.
- `enrollment.rs`: khi ở Enrollment Session thì phát `Connection(Enrolling)` (không phải `Ready`), lấy mã từ `stt` → `Activation { code }`. Audio đọc mã vẫn đi qua `TtsStart…TtsStop` để hub phát được.
- Trong Enrollment Session, `ListenStart`/`Audio`/`DirectSpeech` xử lý như khi chưa Ready (bất biến 2). Cooldown replay 60 s của server: không tự gửi `listen`.
- Đóng `1000` sau claim → kết nối lại ngay, không backoff.
- Timeout tuyệt đối 120 s của server → đóng; kết nối lại theo backoff (ticket 04).

## Nghiệm thu

- [ ] Test: kịch bản enrollment → `Enrolling`, `Activation { code }` đúng mã.
- [ ] Test: server đóng `1000` sau claim → kết nối lại ngay, lần sau server giả trả Voice Session → `Ready`.
- [ ] Test: `DirectSpeech` trong Enrollment Session → `DirectSpeechRejected { reason: Offline }`.
