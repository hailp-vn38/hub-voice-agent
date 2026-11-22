---
status: accepted
---

# Build bản Pi trong container Debian bookworm arm64

Binary cho Pi build trong container `linux/arm64` Debian bookworm (chạy native trên Mac Apple Silicon) để glibc khớp Raspberry Pi OS. Không dùng `cross` hay `cargo-zigbuild`: `libinput`, `libgbm`, EGL cần sysroot đầy đủ, container bookworm có sẵn nên đơn giản hơn.

## Consequences

- Thư mục `target` riêng cho bản Pi (vd. `target/pi`) để không lẫn với bản Mac.
