# 01 — Bridge khung: Nav, StatusBar, Voice, Response, ToolTrail, Kết nối voice

Status: ready-for-human
Blocked by: —
Spec: [../spec.md](../spec.md) · Thiết kế: [../design.md](../design.md) §1 · UI: hub-ui #03, #04, #05, #17 (Kết nối)

## Việc

- Commit riêng, không đổi hành vi: tách `src/ui/bridge.rs` thành `src/ui/bridge/{mod.rs, inputs.rs, voice.rs}`.
- `bridge::spawn(app, ui_rx)`: mỗi lần `UiState` đổi, gọi `hub_core::present` một lần trên luồng tokio, so với `HubView` trước và chỉ gửi mục đã đổi qua `slint::invoke_from_event_loop`. (Tham số `levels_rx` thêm ở #10.)
- `inputs.rs`:
  - `Inputs::send(UserInput)`: `try_send`, log một chỗ khi đầy hoặc đóng.
  - Bảng `Screen` Slint ↔ `hub_core::Screen`, có test phủ mọi biến thể của `hub_core::Screen`. `first-boot` không gửi về core (log và bỏ qua).
- Hợp đồng `contract/nav.slint`: thêm `callback activity()`. Shell gọi khi có chạm hoặc kéo ở bất kỳ đâu. `Inputs` gửi `UserInput::Activity` tối đa 1 lần/giây. Demo bỏ qua callback này.
- `Nav.screen` chỉ set từ `HubView.nav`; `Nav.navigate(s)` → `UserInput::Open(s)`. Bỏ đoạn đổi màn tại UI và comment cũ "hub-core chưa có view".
- `voice.rs` map từ `HubView`:
  - `StatusBar.time`, `StatusBar.offline`; `wifi-strength`, `mic-muted`, `day-phase` giữ mặc định của hợp đồng.
  - `Voice`: `state`, `mic-muted`, `wake-serial`, `tool-serial`, `error-serial`; `orb-tapped` → `UserInput::OrbTapped`.
  - `Response`, `ToolTrail`.
  - `SettingsData.connection` và `ota-url` (giữ quy tắc chỉ ghi URL khi core đổi), `save-voice-url`, `connect-voice` như hiện tại.
- Điền bảng "Map với hub-core" của hub-ui #03, #04, #05 cho các field trên.

## Nghiệm thu

- [x] Test bảng `Screen`: mọi biến thể `hub_core::Screen` có đích Slint và đi về được.
- [x] Test `Inputs`: gọi `activity` 10 lần trong 1 s → gửi 1 `Activity`.
- [ ] Tay: bấm mục Menu → màn đổi (đi qua hub-core); để yên 60 s → về trung tâm; bấm liên tục trên một màn quá 60 s → không bị đưa về.
- [ ] Tay: Settings → Kết nối voice vẫn lưu URL, hiện mã kích hoạt, nút Kết nối hoạt động.
- [ ] Tay (có vela-server): chạm Orb → Orb sang trạng thái nghe; chạm lần nữa → về Idle. (Chưa có mic ở ticket này; nói chuyện thật kiểm ở #08.)
- [x] `scripts/run.sh run --features demo` vẫn chạy như trước.

## Comments

- 2026-11-23: Test `Screen` round-trip và `Activity` burst đạt; `scripts/test.sh -p hub` đạt 145 tests. `cargo check -p hub`, `cargo check -p hub --features demo`, và `cargo clippy --no-deps -p hub --all-targets -- -D warnings` đạt. Demo khởi chạy qua `scripts/run.sh run --features demo` và thoát bằng Ctrl-C. Còn các kiểm tra tay về Menu/timeout, Settings, Orb với vela-server; chưa đánh dấu hoàn tất.
