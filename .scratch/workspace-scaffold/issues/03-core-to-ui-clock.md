# 03 — hub-core phát UiState đồng hồ tới UI

Status: resolved
Blocked by: 02
Spec: [../spec.md](../spec.md)

## Việc

Tracer bullet cho [ADR 0002](../../../docs/adr/0002-single-process-actor-ui-reads-snapshot.md): `hub-core` giữ trạng thái, UI chỉ đọc ảnh chụp.

Trong `hub-core` (chỉ phần tối thiểu của interface trong codebase.md §3.2):

- `ports.rs`: trait `Clock` (`now()`), adapter `SystemClock`.
- `ui_state.rs`: `UiState { clock: String }` (hoặc giờ dạng có cấu trúc, UI tự format — chọn một, ghi lý do trong PR).
- `lib.rs`: `pub async fn run(ports: Ports, io: Io) -> anyhow::Result<()>` với `Ports { clock }` và `Io { ui_tx: watch::Sender<UiState> }`; tick 1 Hz, `send_if_modified`.
- Thư viện thời gian: theo codebase.md dự kiến `jiff`; dùng nó nếu không có lý do khác.

Trong `apps/hub`:

- `main.rs`: tạo tokio runtime trên **thread phụ** chạy `hub_core::run`; Slint chạy trên main thread.
- `ui/bridge.rs`: task đọc `watch::Receiver<UiState>`, đẩy sang Slint bằng `slint::invoke_from_event_loop` (giữ `Weak` của window).
- Đóng cửa sổ → process thoát sạch (runtime dừng).

## Nghiệm thu

- [ ] `cargo run -p hub`: đồng hồ chạy theo giây.
- [ ] Test trong `crates/hub-core/tests/` dùng clock test bám tokio time paused: gửi qua `run`, tiến 1 giây, `UiState.clock` đổi đúng giá trị mong đợi.
- [ ] `cargo test -p hub-core` không build Slint.
- [ ] Clippy/fmt qua.
