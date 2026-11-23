# 03 — Khung app: tầng màn, vuốt, chấm trang, thanh trạng thái

Status: resolved
Blocked by: 02
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn hình", "Điều hướng", "Hiệu ứng theo trạng thái" (chuyển màn)

## Việc

- `app.slint` thành khung: tầng nội dung (màn theo `Nav.screen`), hàng dưới cùng dành cho Voice dock (Orb neo góc phải dưới, ticket 04), tầng lớp phủ (ticket 14). Mỗi màn là placeholder cho tới khi ticket của nó xong.
- **Hai tầng ngang** trung tâm ⇄ Menu bằng `SwipeGestureHandler`: menu trượt vào từ phải **theo ngón tay**, màn trung tâm lùi (scale ~0.92, mờ dần). Nhả quá ~30% chiều rộng hoặc vuốt nhanh → hoàn tất và gọi `Nav.navigate`; không thì bật về. ~400 ms, chỉ transform + opacity.
- Màn tính năng: vuốt trái → phải về Menu, hoặc về màn trung tâm nếu đến bằng đường tắt. Lưu "đến từ đâu" là trạng thái trình bày trong Slint; màn đích vẫn do `Nav.screen` quyết.
- **Vùng cử chỉ:** vuốt bắt đầu trên Orb không tính; cử chỉ bắt đầu trên điều khiển ngang (slider) thuộc điều khiển đó; vuốt dọc không dùng cho điều hướng.
- Chuyển màn không do vuốt (tool, tự về, Ambient): slide/fade 200–300 ms.
- `components/page-dots.slint`: chấm chỉ trang nhỏ mép trên (trung tâm / menu).
- `components/status-bar.slint` + `contract/status.slint`: `StatusBar { time; offline: bool; wifi-strength (0..3); mic-muted: bool; day-phase: float }`; badge offline nhỏ ở góc.
- **Nền Idle đổi màu chậm theo giờ trong ngày** (concept "Hiệu ứng theo trạng thái"): gradient nền nội suy theo `StatusBar.day-phase` (0 = nửa đêm, 0.5 = trưa), cập nhật mỗi phút.

## Demo

Kịch bản: vuốt bằng chuột qua lại; "Mở Thời tiết (tool)" từ màn bất kỳ; "Tự về trung tâm"; "Offline bật/tắt".

## Nghiệm thu

- [ ] Kéo chậm: menu bám theo con trỏ; nhả dưới 30% bật về, trên 30% hoàn tất; vuốt nhanh ngắn cũng hoàn tất.
- [ ] Kéo bắt đầu trên vùng Orb không đổi màn.
- [ ] Orb không di chuyển khi chuyển màn; chỉ nội dung phía trên trượt.
- [ ] `Nav.screen` đổi từ Rust → chuyển màn bằng fade/slide, không cần cử chỉ.
- [ ] Đo fps khi vuốt trên Pi (ready-for-human); ghi số vào Comments.

## Map với hub-core

Map: chờ hub-core #05 (view), #03/#06 (offline, giờ); Wi-Fi chưa có ticket

| Hợp đồng | UiState / UserInput |
|---|---|
| `Nav.screen` | |
| `Nav.navigate(Screen)` | |
| `StatusBar.time` | |
| `StatusBar.offline` | |
| `StatusBar.wifi-strength` | |
| `StatusBar.mic-muted` | |
| `StatusBar.day-phase` | |

## Answer

Làm xong: components/shell.slint (tầng trung tâm/menu/tính năng, vuốt theo ngón tay, ngưỡng 30% hoặc >120px trong <250 ms), page-dots, status-bar, idle-background (gradient theo day-phase), contract/status.slint, screens/placeholder.slint. Demo thêm 3 kịch bản; day_phase có unit test.

Lệch/ghi chú:
- Hàng dock 240 px dưới cùng (cả chiều ngang) bị loại khỏi vùng vuốt, không chỉ ô Orb; chưa có cơ chế khoá cử chỉ cho slider.
- Shell chỉ dùng ở chế độ demo; nhánh không-demo giữ nguyên để app thật vẫn chạy.
- Giờ/day-phase demo theo UTC. Đo fps trên Pi và thử cảm giác vuốt bằng tay chưa làm (ready-for-human).
