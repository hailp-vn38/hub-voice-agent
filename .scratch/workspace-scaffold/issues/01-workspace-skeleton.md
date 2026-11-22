# 01 — Workspace và ba crate rỗng

Status: resolved
Blocked by: —
Spec: [../spec.md](../spec.md)

## Việc

Dựng Cargo workspace theo cây trong [docs/codebase.md §2](../../../docs/codebase.md):

- `Cargo.toml` gốc: `members = ["crates/*", "apps/*"]`, `resolver = "2"` (hoặc mới hơn theo edition), `[workspace.package]` (edition, version), `[workspace.dependencies]` rỗng hoặc chỉ những gì cần ngay.
- `rust-toolchain.toml`: channel `1.98`, component `clippy`, `rustfmt`.
- `.gitignore`: `target/`, `models/`, `.data/`.
- `crates/voice-client`: lib rỗng (`lib.rs` chỉ có doc comment mô tả trách nhiệm).
- `crates/hub-core`: lib rỗng, phụ thuộc `voice-client` (đúng chiều ADR 0001).
- `apps/hub`: package `hub`, `[[bin]] name = "ai-hub"`, `main.rs` in một dòng rồi thoát; phụ thuộc `hub-core`.

Không thêm module, type hay dependency nào chưa dùng.

## Nghiệm thu

- [ ] `cargo build --workspace` và `cargo test --workspace` qua.
- [ ] `cargo clippy --workspace -- -D warnings` và `cargo fmt --check` qua.
- [ ] `cargo run -p hub` chạy được.
- [ ] `cargo tree -p hub-core` chỉ có `voice-client` là crate nội bộ.
