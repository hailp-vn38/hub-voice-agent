# 05 — Build bản Pi trong Docker

Status: resolved
Blocked by: 04
Spec: [../spec.md](../spec.md)

## Việc

Theo [ADR 0011](../../../docs/adr/0011-pi-build-in-docker-bookworm.md) và [development.md §4](../../../docs/development.md):

- `docker/pi-build.Dockerfile`: `debian:bookworm`, `--platform linux/arm64`, rustup đúng phiên bản trong `rust-toolchain.toml`, thư viện: `libudev-dev libinput-dev libdrm-dev libgbm-dev libegl-dev libgles-dev libxkbcommon-dev libfontconfig-dev pkg-config` (bỏ `libasound2-dev` tới khi có audio; thêm thư viện nào còn thiếu thì ghi lại).
- `scripts/build-pi.sh`: dựng image nếu chưa có, chạy `cargo build --release -p hub --no-default-features --features pi` trong container; mount repo, cache `~/.cargo/registry` qua volume, `CARGO_TARGET_DIR=target/pi`.
- Cập nhật development.md §4 nếu lệnh hoặc danh sách thư viện thực tế khác dự kiến.

## Nghiệm thu

- [ ] `scripts/build-pi.sh` chạy từ đầu trên Mac Apple Silicon (Docker 28) và thành công.
- [ ] `file target/pi/release/ai-hub` là `ELF 64-bit LSB … ARM aarch64`.
- [ ] Lần build thứ hai dùng cache, không tải lại crate.
- [ ] Build Mac (`target/`) không bị ảnh hưởng.
