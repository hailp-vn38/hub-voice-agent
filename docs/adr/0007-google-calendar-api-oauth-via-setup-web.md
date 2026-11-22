---
status: accepted
---

# Google Calendar API, kết nối OAuth bằng cách dán URL qua Setup web

Lịch đọc và tạo đều qua Google Calendar API (scope `calendar.events`, lịch `primary`), bỏ iCal: dữ liệu thời gian thực, sự kiện vừa tạo hiện ngay, một nguồn duy nhất. Làm mới mỗi 5 phút bằng `syncToken` và ngay sau mỗi lần tạo; không dùng push notification của Google (cần webhook HTTPS công khai).

OAuth device flow không hỗ trợ scope Calendar và Google không nhận redirect về IP LAN, nên kết nối một lần qua Setup web: client loại "Desktop app", redirect `http://127.0.0.1:<port>`; người dùng chép URL trang lỗi `127.0.0.1` trên điện thoại dán vào Setup web, hub đổi mã lấy refresh token.

## Consequences

- App Google phải ở trạng thái **published** (Testing làm refresh token hết hạn sau 7 ngày); người dùng sẽ thấy cảnh báo "ứng dụng chưa được xác minh".
- Trên Mac, trình duyệt cùng máy nên redirect về thẳng app, không cần bước dán URL.
