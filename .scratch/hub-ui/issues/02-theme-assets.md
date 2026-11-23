# 02 — Theme, font, icon

Status: ready-for-human
Blocked by: 01
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Nguyên tắc", "Phong cách"

## Việc

- `ui/theme.slint`: global `Theme` gồm token màu (nền tối, chữ, chữ nhạt, nhấn voice, ấm, lỗi, xanh hoàn tất, nền thẻ), cỡ chữ theo bậc (giờ/nhiệt độ lớn nhất, tiêu đề, thân, phụ), khoảng cách, bo góc, thời lượng animation chuẩn (chuyển màn ~400 ms, nảy ~600 ms, rung chữ ~550 ms).
- `in property <float> text-scale` (nhỏ / vừa / lớn) nhân vào mọi cỡ chữ; Setting (#17) ghi giá trị này.
- `Theme.touch-min: 88px` cho vùng chạm.
- Font Be Vietnam Pro (OFL) hai độ đậm trong `assets/fonts/`, nạp bằng `import "…ttf"` để nhúng; kiểm dấu tiếng Việt đầy đủ.
- Icon Tabler outline (MIT) dạng SVG trong `assets/icons/`; component `components/icon.slint` (`Icon { name; size; color; }`) gom đường dẫn một chỗ. Ghi nguồn và giấy phép trong `assets/README.md`.
- Sửa `app.slint` và `screens/voice-settings.slint` hiện có sang dùng token mới (không đổi hành vi).

## Nghiệm thu

- [ ] Demo panel có kịch bản "Bảng mẫu": hiện mọi bậc chữ tiếng Việt có dấu ("Hẹn giờ luộc trứng 7 phút · Thứ Bảy"), mọi màu, các icon đã dùng.
- [ ] Đổi `text-scale` ở kịch bản → mọi chữ đổi cỡ, layout không tràn ở 1920×1080.
- [ ] Trên Pi: font hiển thị đúng dấu qua FemtoVG (ready-for-human).

## Map với hub-core

Map: chờ hub-core (Setting cỡ chữ, chưa có ticket)

| Hợp đồng | UiState / UserInput |
|---|---|
| `Theme.text-scale` | |

## Answer

Đã thêm `Theme` (màu, cỡ chữ nhân `text-scale`, spacing, animation, `touch-min`), font Be Vietnam Pro nhúng, 10 icon Tabler + `Icon`, `assets/README.md`, màn `screens/sample.slint`. Demo panel: "Bảng mẫu", "Chữ nhỏ/vừa/lớn". `app.slint` và `voice-settings.slint` dùng token. Build demo OK; chưa kiểm bằng mắt, chưa kiểm trên Pi.

## Comments

- 2026-11-23 (review hub-ui): Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
