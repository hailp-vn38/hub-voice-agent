# 04 — Feature `pi`: backend linuxkms

Status: resolved
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

- Thêm feature `pi` cho `apps/hub`: `slint/backend-linuxkms` + `slint/renderer-femtovg` (theo [ADR 0003](../../../docs/adr/0003-slint-linuxkms-femtovg.md)).
- `desktop` và `pi` loại trừ nhau: nếu bật cả hai hoặc không bật cái nào thì `compile_error!` với thông báo rõ.
- Không có code nhánh theo feature ngoài phần chọn backend.
- Ghi vào [docs/development.md](../../../docs/development.md) nếu tên feature Slint thực tế khác với dự kiến (vd. `backend-linuxkms-noseat`) và lý do chọn.

## Nghiệm thu

- [ ] `cargo build -p hub` (mặc định `desktop`) vẫn qua trên Mac.
- [ ] `cargo build -p hub --features pi` báo `compile_error!` do bật cả hai.
- [ ] `cargo check -p hub --no-default-features` báo `compile_error!` do không bật cái nào.

Build thật với `--no-default-features --features pi` được kiểm ở ticket 05 trong container (trên Mac thiếu thư viện hệ thống).
