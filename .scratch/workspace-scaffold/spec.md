# Spec: Dựng workspace

Status: ready-for-agent

## Mục tiêu

Dựng khung repo theo [docs/codebase.md](../../docs/codebase.md) và kiểm chứng sớm hai chuỗi rủi ro nhất: **build bản Pi trong Docker** và **Slint `linuxkms` chạy trên Pi thật**. Ứng với bước 1–2 trong [development.md §7](../../docs/development.md).

Xong spec này thì:

- `cargo build` và `cargo test` chạy trên Mac cho cả workspace.
- `cargo run -p hub` mở một cửa sổ Slint trên Mac, hiện đồng hồ do `hub-core` cập nhật mỗi giây qua `UiState`.
- Lệnh build Pi tạo binary `aarch64-unknown-linux-gnu`; binary đó chạy bằng systemd trên Pi và hiện cùng màn đó toàn màn hình, không X11/Wayland.

## Trong phạm vi

| Hạng mục | Chi tiết |
|---|---|
| Workspace | `Cargo.toml` gốc với `[workspace.dependencies]`, `rust-toolchain.toml` (1.98), `.gitignore` (`target/`, `models/`, `.data/`) |
| `crates/voice-client` | Crate lib rỗng, build được |
| `crates/hub-core` | `run(...)` tối thiểu: phát `UiState { clock }` mỗi giây qua `watch`, đọc giờ qua port `Clock` |
| `apps/hub` | Package `hub`, binary `ai-hub`; feature `desktop` (mặc định, `backend-winit`) và `pi` (`backend-linuxkms` + `renderer-femtovg`); `build.rs` dùng `slint-build`; `ui/app.slint` + `ui/theme.slint` |
| Nối dây | tokio runtime trên thread phụ chạy `hub_core::run`; Slint event loop trên main thread; bridge `UiState` → property Slint ([ADR 0002](../../docs/adr/0002-single-process-actor-ui-reads-snapshot.md)) |
| Build Pi | `docker/pi-build.Dockerfile` (bookworm, `linux/arm64`) + script build, `target/pi` riêng ([ADR 0011](../../docs/adr/0011-pi-build-in-docker-bookworm.md)) |
| Deploy | `deploy/ai-hub.service`, `scripts/deploy-pi.sh <host>` |

## Ngoài phạm vi

- Mọi tính năng: voice, audio, wake word, widget, Setup web, SQLite, config.
- `Io`/`Ports` đầy đủ như codebase.md: chỉ thêm `ui_tx` và `Clock`, phần còn lại thêm khi tính năng cần.
- Font nhúng, icon, theme hoàn chỉnh; chỉ cần nền tối và chữ đồng hồ lớn.
- CI.
- Kiosk hoàn chỉnh (splash, ẩn console, tắt `getty@tty1`): chỉ cần service chạy được.

## Quyết định cho spec này

- **Tên:** package `hub` (đúng lệnh `-p hub` trong development.md), binary `ai-hub` (đúng tên service).
- **Phiên bản phụ thuộc:** chọn bản ổn định mới nhất lúc làm, khai báo một lần trong `[workspace.dependencies]`, crate con dùng `workspace = true`.
- **Màn hình:** cửa sổ logic 1920×1080 trên Mac; thu nhỏ bằng `SLINT_SCALE_FACTOR`.
- **Đồng hồ** chứng minh luồng `hub-core → watch → bridge → Slint`, không phải widget đồng hồ thật: chỉ `HH:MM:SS`, múi giờ hệ thống.

## Tiêu chí nghiệm thu

1. Trên Mac: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings` đều qua.
2. `cargo test -p hub-core` không build Slint (kiểm bằng `cargo tree -p hub-core` không có `slint`).
3. Có test trong `hub-core` dùng clock test + tokio time paused, kiểm `UiState.clock` đổi sau khi tiến 1 giây.
4. `cargo run -p hub` mở cửa sổ, đồng hồ chạy theo giây.
5. Build Pi bằng một lệnh, ra file ELF aarch64 (`file target/pi/.../ai-hub`).
6. Trên Pi thật: `systemctl start ai-hub` hiện đồng hồ toàn màn hình qua DRM/KMS; `journalctl -u ai-hub` không lỗi; ghi lại fps/độ trễ khởi động nếu đo được.

## Tickets

| # | Ticket | Blocked by | Status |
|---|---|---|---|
| 01 | [Workspace và ba crate rỗng](issues/01-workspace-skeleton.md) | — | resolved |
| 02 | [Cửa sổ Slint trên Mac](issues/02-slint-window-desktop.md) | 01 | resolved |
| 03 | [hub-core phát UiState đồng hồ tới UI](issues/03-core-to-ui-clock.md) | 02 | resolved |
| 04 | [Feature `pi`: backend linuxkms](issues/04-pi-feature-linuxkms.md) | 02 | resolved |
| 05 | [Build bản Pi trong Docker](issues/05-pi-docker-build.md) | 04 | resolved |
| 06 | [Deploy và chạy trên Pi thật](issues/06-deploy-on-pi.md) | 03, 05 | ready-for-human |
