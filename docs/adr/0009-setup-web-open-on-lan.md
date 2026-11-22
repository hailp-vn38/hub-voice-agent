---
status: accepted
---

# Setup web luôn mở cho LAN, không xác thực

Setup web là HTTP server nhỏ trong cùng process, luôn mở cho mạng LAN và không có xác thực — quyết định có chủ ý vì thiết bị dùng trong mạng nhà và màn hình không có bàn phím để đặt mật khẩu. Hệ quả chấp nhận: mọi máy trong LAN đều xem và sửa được setting, kể cả token.

## Consequences

- Không log giá trị token; file config chỉ user của app đọc được (0600).
