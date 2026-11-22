# 03 — Enrollment Session và mã kích hoạt

Status: needs-info
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

## Comments

Audit giao thức theo source hiện tại của `AI_Agent_Voice_rust-infapi`:

- OTA không đưa ra discriminant ổn định cho loại session. Với enrollment qua WebSocket, Unknown/Pending nhận `websocket` URL/token nhưng không có `activation` (`crates/vela-server/src/app/enrollment/ota.rs:50-89`, `:203-260`; schema ở `crates/vela-server/src/protocol/server.rs:37-70`). Registered cũng có URL/token; `push` chỉ có khi tính năng tương ứng bật nên không thể dùng sự có mặt/vắng mặt của trường này để phân loại.
- Admission phân Pending sang enrollment handler, còn Registered sang Voice (`crates/vela-server/src/app/enrollment/mod.rs:36-79`, `crates/vela-server/src/app/transport/websocket/admission.rs:255-266`). Tuy vậy Enrollment và Voice đều gửi cùng `ServerHello::v1` với `type`, `transport`, `session_id`, `audio_params` (`crates/vela-server/src/app/enrollment/socket.rs:87-90`, `crates/vela-server/src/app/transport/websocket/voice_session.rs:191-193`, `crates/vela-server/src/protocol/server.rs:3-31`).
- Enrollment chỉ gửi frame `stt` sau handshake và sau khi trạng thái enrollment được kiểm tra/clip được encode (`crates/vela-server/src/app/enrollment/socket.rs:92-105`, `:135-147`). `text` là câu tự do có mã nhúng (`Mã kết nối: …`), không có field `code` hay event type riêng. TTS theo sau bằng `tts:start`, `tts:sentence_start` (có `text`) và raw binary Opus, rồi `tts:stop` (`:141-156`). Các frame đầu cũng có thể là thông báo terminal `stt` nếu trạng thái đã đổi (`:258-276`). Dò text hoặc trích số từ câu không phải tín hiệu phân loại/activation có cấu trúc.
- Có vấn đề thứ tự: client cần quyết định `Connection(Ready)` so với `Connection(Enrolling)` quanh `ServerHello`, nhưng `stt` phân biệt được chỉ đến sau đó. Nếu phát `Ready` ngay khi nhận hello thì Enrollment phát ra Ready nhầm; nếu chờ đoán từ frame tùy chọn sau hello thì không có ranh giới/protocol marker đảm bảo để phát Ready đúng lúc. Claim gửi `stt` rồi close `1000` (`:258-276`); timeout tuyệt đối cũng close `1000` (`:96-100`, `:121-127`), nên close code tự nó không phân biệt claim với timeout.

Cần quyết định thay đổi wire contract phía server trước khi implement: thêm `session_kind` rõ ràng (`voice`/`enrollment`) vào ServerHello (bao gồm cách tương thích client cũ), và một field/frame activation có cấu trúc như `activation.code` thay cho parse `stt.text`; hoặc định nghĩa một frame Enrollment mở đầu bắt buộc đến trước khi client phát bất kỳ `Ready` nào. Sau khi contract được chốt, có thể implement và test qua public channels mà không dựa vào text matching.
