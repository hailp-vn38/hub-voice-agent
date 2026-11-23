# 18 — Ambient

Status: ready-for-human
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn hình" (Ambient), "Hiệu ứng theo trạng thái" (Idle), Orb Idle/Ambient

## Việc

- `screens/ambient.slint`: đồng hồ lớn + nhiệt độ, tông tối và độ tương phản thấp; Orb thở nhỏ và mờ.
- Đồng hồ chuyển số nhẹ (mỗi phút); dịch vị trí chậm theo giờ để tránh lưu ảnh trên màn.
- Chạm bất kỳ đâu → `AmbientData.wake()`; wake word thoát do hub-core đổi `Nav.screen`.
- Lớp báo (ticket 14) và ô mốc "Hôm nay là …" vẫn hiện được trên Ambient.
- Độ sáng đèn nền thật do hub-core qua `System`; UI chỉ làm tối nội dung.
- `contract/ambient.slint`: `global AmbientData { in property <string> time; in property <string> temp; in property <string> icon; in property <bool> night; callback wake(); }`.

## Demo

Kịch bản: "Vào Ambient", "Ambient ban đêm", "Chạm để thoát", "Hết giờ khi đang Ambient".

## Nghiệm thu

- [ ] Vào/ra Ambient bằng fade, không giật.
- [ ] Đọc được giờ từ xa trong phòng tối.
- [ ] Chạm thoát; báo hiện trên Ambient.

## Map với hub-core

Map: done (hub-core #21; bridge ticket 06)

| Hợp đồng | UiState / UserInput |
|---|---|
| `AmbientData.time` / `.temp` / `.icon` | Chép từ lớp `present` của `HubView.ambient`; giờ và thời tiết được suy ra từ `UiState` (#21) |
| `.night` | `HubView.ambient.night`, hub-core suy ra từ dự báo ngày hiện tại (#21) |
| `wake()` | `UserInput::Activity`; hub-core thoát Ambient về Center |

## Answer

Added `contract/ambient.slint` (AmbientData), `screens/ambient.slint` (large dim clock, temp+icon, slow hourly drift via day-phase, small breathing dot as Orb stand-in, tap -> wake()), fade overlay in `shell.slint`, demo scenarios "Vào Ambient" / "Ambient ban đêm", demo wake -> center. Overlay layer (14) and "Hôm nay" tile are not yet present; Ambient sits below the top layer so they can show. Real Orb replaces the dot in ticket 04.

## Comments

- 2026-11-23 (review hub-ui): Bỏ chấm thở tạm thay Orb; Orb thật của Shell hiện trên Ambient, thu nhỏ và mờ. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
