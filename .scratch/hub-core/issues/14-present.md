# 14 — Present: lớp trình bày cho hợp đồng UI

Status: ready-for-agent
Blocked by: 05
Spec: [../spec.md](../spec.md) · UI: [../../hub-ui/spec.md](../../hub-ui/spec.md) (Quy tắc hợp đồng 1–3)

## Vì sao

`UiState` chứa dữ liệu miền thô (giây, chuỗi RFC 3339, JSON). Hợp đồng Slint cần dữ liệu đã sẵn để hiển thị: chuỗi tiếng Việt, `progress` 0..1, `key` ổn định, `serial`. Spec hub-ui cấm nhánh nghiệp vụ trong bridge, nên phần suy ra phải nằm trong hub-core, sau một interface nhỏ và test được không cần Slint.

## Việc

- `src/present/`: `pub fn present(state: &UiState) -> HubView`, hàm thuần, không I/O. `HubView` là bản sao Rust của các global hợp đồng (`apps/hub/ui/contract/*.slint` trên `feat/hub-ui`); mỗi tính năng một submodule, thêm dần theo ticket 15–21.
- `UiState.now: jiff::Zoned` ghi mỗi tick (thay cho việc chỉ có `clock: String`); `present` không đọc đồng hồ hệ thống.
- `UiState.events`: bộ đếm `u64` cho hiệu ứng một lần (ADR 0002): `focus`, `wake`, `tool`, `error`, `alert`, `confetti`, `spawn`. Chỉ tăng, không reset.
- `View` tăng `events.focus` mỗi lần `open`/`show` đặt focus, kể cả khi focus trùng focus cũ.
- Lát đầu của `HubView`: `nav` (màn hiện tại, map 1–1 `Screen`), `status` (`time` "HH:MM", `offline` theo `VoiceState::Offline`), `ambient` (`time`, `temp`, `icon`, `night`).
- `UserInput::Focus(Option<Focus>)`: chạm chọn ngày / ảnh đổi focus trên màn hiện tại, không đổi màn, không tăng `events.focus` (animation focus chỉ cho thay đổi do giọng nói).
- Export `present`, `HubView` từ `lib.rs`.

## Nghiệm thu

- [ ] Test trong `tests/present.rs`: dựng `UiState` mẫu → so `HubView.nav/status/ambient`.
- [ ] Test: `hub_timer_start` hai lần cùng thẻ → `events.focus` tăng hai lần.
- [ ] `present` không có tham số đồng hồ; test dùng `now` cố định.
- [ ] Test: `UserInput::Focus` đổi `view.focus`, giữ `view.screen`, không tăng `events.focus`.

## Comments

- 2026-11-23, người dùng chốt: trạng thái xem phụ nào tool giọng nói lái được thì hub-core giữ qua `Focus`; còn lại UI giữ tạm trong adapter. Cụ thể:
  - hub-core: ngày chọn của Lịch (`hub_calendar_get` `focus`), ngày chọn của Thời tiết (`hub_weather_get` `focus`), ảnh đang xem trong viewer (`hub_show_image`, `hub_image_control`).
  - UI: `toggle-month`, `open-detail`/`close-detail` của Lịch, `select`/`open-detail` của Đếm ngược, `auto-advance` của viewer.
  - Chạm vào trạng thái do hub-core giữ gửi `UserInput::Focus`, không đổi tại UI.
- Bước map ở `apps/hub` cần gộp `feat/hub-core` vào `feat/hub-ui` trước (nhánh UI tách từ hub-core cũ, `Ports` chỉ có `clock`). Việc gộp và bridge thuộc ticket hub-ui.
