# 08 — Menu tính năng

Status: ready-for-human
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Điều hướng" (Menu tính năng)

## Việc

- `screens/menu.slint`: lưới 4 × 2 ô lớn theo thứ tự Thời tiết, Lịch, Nhà, Hẹn giờ, Đếm ngược, Reminder, Ảnh, Setting. Mỗi ô: icon, tên, một dòng thông tin sống.
- `contract/menu.slint`: `global Menu { in property <string> weather; in property <string> calendar; in property <string> home; in property <string> timers; in property <string> countdowns; in property <string> reminders; in property <string> photos; in property <string> settings; }` — mỗi field là dòng thông tin sống đã định dạng ("1 đang chạy", "3 sự kiện hôm nay").
- Chạm ô → `Nav.navigate(screen)`. Vuốt trái → phải về màn trung tâm (khung ở ticket 03).
- Không lồng sâu hơn một tầng.

## Demo

Fixture: đủ 8 dòng; kịch bản "Không có gì đang chạy" (dòng trống hoặc câu mặc định).

## Nghiệm thu

- [ ] 8 ô vừa màn phía trên Voice dock, mỗi ô ≥ `Theme.touch-min`.
- [ ] Chạm từng ô mở đúng màn; vuốt về menu rồi về trung tâm.

## Map với hub-core

Map: done (hub-core #21; bridge ticket 06)

| Hợp đồng | UiState / UserInput |
|---|---|
| `Menu.weather` … `Menu.settings` | Chép trường tương ứng từ `HubView.menu` (`present` #21); hub-core tạo từng dòng từ `UiState` và view của tính năng. |

## Answer

Đã thêm `contract/menu.slint` (global Menu), `screens/menu.slint` (lưới 4x2, ô chạm gọi `Nav.navigate`, chừa chỗ dock), nối vào Shell thay Placeholder menu. Vuốt về trung tâm dùng khung ở ticket 03. Fixture `fixtures/menu.rs` và 2 kịch bản demo. Chưa kiểm tra bằng mắt.

## Comments

- 2026-11-23 (review hub-ui): Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
