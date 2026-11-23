# 16 — Màn Nhà

Status: ready-for-human
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §5 "Home Assistant (demo)", §6 "Màn hình" (Nhà), "Phong cách" (màu ấm cho thiết bị đang bật)

## Việc

- `screens/home.slint`: "‹ Nhà"; toàn bộ thẻ demo đã chọn: đèn, công tắc (chạm để bật/tắt), cảm biến (chỉ đọc).
- Thẻ đang bật dùng màu ấm; đang chờ phản hồi sau khi chạm thì hiện trạng thái chờ; `stale` → thẻ mờ + "cập nhật lúc HH:MM"; mất kết nối HA → dòng báo ở đầu màn.
- Chưa cấu hình HA → trạng thái trống hướng dẫn mở Setup web.
- `components/home-tile.slint` dùng chung với màn trung tâm (ticket 07).
- `contract/home.slint`:
  - `export enum HomeKind { light, switch, sensor }`.
  - `HomeTile { key, name, kind: HomeKind, on: bool, value, pending: bool, stale: bool }`.
  - `global HomeData { in property <[HomeTile]> tiles; in property <bool> configured; in property <bool> connected; in property <string> updated; callback toggle(string); }`.

## Demo

Kịch bản: "Bật đèn phòng khách" (chờ → bật), "Mất kết nối HA" (stale), "Chưa cấu hình".

## Nghiệm thu

- [ ] Đèn/công tắc bật tắt bằng chạm, trạng thái chờ hiện rõ.
- [ ] Cảm biến không chạm được.
- [ ] Thẻ cũ phân biệt rõ với thẻ đang cập nhật.

## Map với hub-core

Map: done · hub-core #11; hub-mac #05, commit `9c100af`

| Hợp đồng | UiState / UserInput |
|---|---|
| `HomeData.tiles` | `HubView.home.tiles` |
| `.configured` / `.connected` / `.updated` | `HubView.home.configured` / `.connected` / `.updated` |
| `toggle(key)` | `UserInput::ToggleHome(key)` |

## Answer

Done: contract/home.slint, components/home-tile.slint (HomeTileView), screens/home.slint wired in Shell for Screen.home, demo fixtures/home.rs and 3 scenarios. Pending tiles show "Đang chờ…", stale tiles dim with "cập nhật lúc HH:MM", disconnect banner and unconfigured empty state. No Rust logic to unit-test; verified by build only.

## Comments

- 2026-11-23 (review hub-ui): Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
