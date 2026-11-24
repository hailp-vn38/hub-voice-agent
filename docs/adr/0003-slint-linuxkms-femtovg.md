---
status: accepted
---

# Slint vẽ thẳng ra DRM/KMS trên máy tính bảng, winit trên Mac

UI dùng Slint với backend `linuxkms` và renderer FemtoVG (OpenGL ES) trên máy tính bảng Debian 13, không X11/Wayland, để boot thẳng vào app và không tốn tài nguyên cho compositor. Trên Mac dùng `backend-winit`. Chọn qua feature của `apps/hub`: `desktop` (mặc định) và `kiosk`.

## Consequences

- Hiệu ứng chỉ dùng transform, opacity, gradient; không blur toàn màn (GPU tích hợp yếu). Đo fps trên máy thật trước khi chốt hiệu ứng.
- Backend, cảm ứng thật và fps chỉ kiểm chứng được trên máy tính bảng.
