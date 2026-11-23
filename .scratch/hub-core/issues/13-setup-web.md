# 13 — Setup web

Status: ready-for-agent
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

- `src/setup_web/`: `serve(addr, …)` mở trên LAN theo ADR 0009; trang HTML tĩnh để nhập OTA URL, kết nối Google (OAuth callback), HA, vị trí nhà.
- Ghi qua `ConfigStore::update`; thay đổi tự lan tới actor qua `watch()`.

## Nghiệm thu

- [ ] Test HTTP trên cổng ngẫu nhiên: POST OTA URL → `voice_config_tx` đổi.
- [ ] OAuth callback đổi mã → refresh token được lưu (fake Google).

## Comments

- Cần chốt khi triage: xác thực Setup web trên LAN (mã hiển thị trên màn hình?) và phạm vi trang cho first-boot (ADR 0010).
- 2026-11-23: Theo quyết định của người dùng, Setup web không xác thực trên LAN theo ADR 0009. First-boot AP và hai mã QR theo ADR 0010 để ticket riêng, ngoài phạm vi ticket này.
