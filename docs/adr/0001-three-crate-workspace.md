---
status: accepted
---

# Workspace ba crate: voice-client, hub-core, apps/hub

Repo là một Cargo workspace gồm `crates/voice-client` (Voice Protocol, không UI, không thiết bị âm thanh), `crates/hub-core` (toàn bộ hành vi của hub, không Slint, không thiết bị âm thanh, không lời gọi OS) và `apps/hub` (Slint, audio, tích hợp OS, nối dây). Tách `hub-core` để `cargo test -p hub-core` chạy nhanh trên Mac mà không phải build Slint, `cpal`, libopus và ONNX Runtime; quy tắc phụ thuộc là một chiều `voice-client ← hub-core ← apps/hub`.

Type Voice Protocol định nghĩa lại trong `voice-client`, không dùng chung crate với vela-server; chỉ tách crate giao thức dùng chung khi giao thức đã ổn định.

## Consequences

- Thêm code vào `hub-core` mà kéo theo thư viện hệ thống (ALSA, DRM, EGL…) là phá quy tắc này; code đó thuộc `apps/hub`, đặt sau một port.
