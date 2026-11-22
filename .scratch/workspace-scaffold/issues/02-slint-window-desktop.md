# 02 — Cửa sổ Slint trên Mac

Status: resolved
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- Thêm `slint` vào `[workspace.dependencies]` với `default-features = false`; `apps/hub` bật các feature cần qua feature của crate:
  - `desktop` (mặc định): `backend-winit` + một renderer chạy trên Mac.
  - Chưa thêm feature `pi` (ticket 04).
- `apps/hub/build.rs` dùng `slint-build` biên dịch `ui/app.slint`.
- `ui/theme.slint`: vài token màu (nền tối, chữ, màu nhấn) dùng `global`.
- `ui/app.slint`: cửa sổ logic 1920×1080, nền tối, một property `clock: string` hiển thị chữ lớn ở giữa (giá trị mặc định `"--:--:--"`).
- `main.rs`: tạo window, `run()`.

## Nghiệm thu

- [ ] `cargo run -p hub` mở cửa sổ nền tối với `--:--:--`.
- [ ] `SLINT_SCALE_FACTOR=0.5 cargo run -p hub` thu nhỏ cửa sổ, layout không vỡ.
- [ ] `cargo tree -p hub-core` vẫn không có `slint`.
- [ ] Clippy/fmt qua.
