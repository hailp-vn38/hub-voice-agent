# 06 — Thẻ sinh ra từ Orb, thẻ nổi

Status: resolved
Blocked by: 04
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Concept voice-first" mục 4 (spawn choreography); "Agent điều khiển hub" (`hub_show_card`, ví dụ "tạo sự kiện")

## Việc

Mọi kết quả agent tạo ra đi theo thứ tự **Orb → thẻ → chỗ của nó**. Ticket này làm phần dùng chung; mỗi màn tính năng (09–15) dùng lại.

- `contract/spawn.slint`:
  - `export enum PendingState { none, saving, saved, failed }`.
  - `global Spawn { in property <int> serial; in property <string> key; }`: mục có `key` trùng sẽ xuất hiện bằng hiệu ứng sinh từ Orb khi `serial` đổi.
  - `struct FloatingCard { key: string, title: string, body: string, icon: string }`; `global FloatingCards { in property <[FloatingCard]> cards; in property <bool> fading; }` cho `hub_show_card` không thuộc tính năng nào.
- `components/spawn.slint`: `SpawnIn` bọc một thẻ: khi được kích hoạt, thẻ bắt đầu ở toạ độ `Dock.orb-x/orb-y`, nảy (overshoot ~600 ms) rồi bay vào đúng chỗ của nó trong layout. Thẻ khác dời chỗ mượt.
- Trạng thái chờ dùng chung: `saving` (thẻ đã có nội dung, chỉ báo đang lưu), `saved` (dấu check rồi bay vào chỗ), `failed` (rung nhẹ, viền đỏ, ở lại cho tới khi mục bị bỏ).
- `components/floating-cards.slint`: thẻ nổi phía trên Voice dock, mờ đi khi `fading`.
- `components/card.slint`: khung thẻ cơ bản (nền, bo góc, padding theo Theme) cho các màn dùng.

## Demo

Một màn thử chỉ có trong demo: lưới 6 ô; kịch bản "Sinh vào ô 3", "Lưu → thành công", "Lưu → lỗi", "Thẻ nổi", "Thẻ nổi mờ".

## Nghiệm thu

- [ ] Thẻ luôn xuất phát từ Orb, kể cả khi màn vừa chuyển trong cùng lượt.
- [ ] Ghi `Spawn` hai lần với cùng `serial` → chỉ một lần hiệu ứng.
- [ ] `failed`: thẻ rung một lần, viền đỏ, không bay vào chỗ.
- [ ] Không có thẻ nào "tự hiện ra" mà không qua `SpawnIn` ở các màn dùng nó (kiểm lại ở ticket màn).

## Map với hub-core

Map: chờ hub-core #04 trở đi (serial theo tool); `hub_show_card` chưa có ticket hub-core

| Hợp đồng | UiState / UserInput |
|---|---|
| `Spawn.serial` / `.key` | |
| `FloatingCards.cards` | |
| `FloatingCards.fading` | |

## Answer

Done: contract/spawn.slint, components/{card,spawn,floating-cards}.slint, screens/spawn-demo.slint (demo-only SpawnDemo global + 6-cell grid), 5 demo scenarios. SpawnIn uses easeOutBack 600ms from Dock.orb; it runs on serial change or when the card is created in that turn (init), so re-entering a screen with the same key replays it. Check mark is the "✓" glyph (no check icon yet). Not visually verified.
