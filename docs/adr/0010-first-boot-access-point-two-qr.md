---
status: accepted
---

# Lần đầu khởi động: tự phát Wi-Fi và hiện hai mã QR

Setup web cần LAN, mà nhập mật khẩu Wi-Fi trên màn cảm ứng lại cần gõ chữ. Khi chưa có Wi-Fi (và không có Ethernet), hub tự phát một điểm truy cập (vd. "AI-Hub-Setup") và hiện hai mã QR: QR 1 để điện thoại vào mạng đó, QR 2 mở Setup web. Nhập Wi-Fi nhà xong, hub chuyển sang mạng nhà và hiện IP mới.

## Consequences

- Phát điểm truy cập là lời gọi OS (NetworkManager), nằm sau port `System`; trên Mac là adapter giả.
