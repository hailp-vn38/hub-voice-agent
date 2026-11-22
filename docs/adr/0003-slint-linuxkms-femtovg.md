---
status: accepted
---

# Slint vẽ thẳng ra DRM/KMS trên Pi, winit trên Mac

UI dùng Slint với backend `linuxkms` và renderer FemtoVG (OpenGL ES) trên Pi 4, không X11/Wayland, để boot thẳng vào app và không tốn tài nguyên cho compositor. Trên Mac dùng `backend-winit`. Chọn qua feature của `apps/hub`: `desktop` (mặc định) và `pi`.

## Consequences

- Hiệu ứng chỉ dùng transform, opacity, gradient; không blur toàn màn (GPU V3D yếu). Đo fps trên Pi thật trước khi chốt hiệu ứng.
- Backend, cảm ứng thật và fps chỉ kiểm chứng được trên Pi.
