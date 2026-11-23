# 12 — Màn Đếm ngược

Status: ready-for-human
Blocked by: 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Đếm ngược"

## Việc

- `screens/countdowns.slint` + `components/confetti.slint`.
- **Đầu màn:** "‹ Đếm ngược · N mốc".
- **Trái — mốc chính:** nhãn, giá trị rất lớn, ngày đích (kèm ngày âm lịch nếu là mốc âm), thanh tiến độ "X% chặng đường".
- **Phải — các mốc khác:** nhãn, giá trị, ngày đích, "hằng năm" nếu lặp.
- **Trống:** "Hey Jarvis, đếm ngược đến Tết".
- **Đơn vị hiển thị** đến từ hợp đồng (`unit`): `days` ("N ngày"), `live` (HH:MM:SS chạy trực tiếp, màu ấm, cập nhật mỗi giây), `today` ("Hôm nay là {nhãn}"), `passed` ("Đã qua").
- **Hiệu ứng:**
  - Mốc mới: `SpawnIn` từ Orb vào danh sách; nếu thành mốc chính thì thay chỗ, mốc cũ trượt sang danh sách.
  - Số ngày giảm (qua nửa đêm): lật (flip) xuống một.
  - Chạm mốc ở danh sách → phóng lên vị trí chính (`select(key)`).
  - `confetti-serial` đổi → ~30 mảnh pháo giấy bắn ra từ Orb + "Hôm nay là {nhãn}".
  - Xoá: ô mờ dần, các ô khác dồn lên.
- **Chạm mốc chính** → thẻ chi tiết có nút **Xoá** (`remove(key)`). Không nhấn giữ.
- `contract/countdowns.slint`:
  - `export enum CountdownUnit { days, live, today, passed }`.
  - `Countdown { key, label, value, unit: CountdownUnit, target, lunar, repeat-text, progress: float, progress-text }`.
  - `global CountdownsData { in property <string> title; in property <Countdown> main; in property <bool> has-main; in property <[Countdown]> others; in property <bool> detail-open; in property <int> confetti-serial; in property <string> focus; in property <int> focus-serial; callback select(string); callback open-detail(); callback close-detail(); callback remove(string); }`.

## Demo

Kịch bản: "Đếm ngược đến Tết" (âm lịch, spawn, thay mốc chính), "Sinh nhật 12/3", "Qua nửa đêm" (flip), "Còn < 24 giờ" (live), "Hôm nay là Tết" (pháo giấy), "Chuyến đi đã qua", "Còn bao nhiêu ngày đến Tết?" (focus), "Xoá chuyến đi".

## Nghiệm thu

- [ ] Bốn đơn vị hiển thị đúng; `live` đổi mỗi giây.
- [ ] Flip chỉ chạy khi số ngày giảm, không chạy lúc mở màn.
- [ ] Pháo giấy xuất phát từ Orb, ~30 mảnh, tự dọn.
- [ ] Đo fps pháo giấy trên Pi (ready-for-human).

## Map với hub-core

Map: chờ hub-core #07

| Hợp đồng | UiState / UserInput |
|---|---|
| `CountdownsData.title` | `HubView.countdowns.title` |
| `.main` / `.has-main` / `.others` | `HubView.countdowns.main`, `.has_main`, `.others`; bridge đưa key được chọn lên `main`, mốc chính cũ vào danh sách |
| `.detail-open` | Trạng thái bridge từ `open-detail` / `close-detail`, mặc định `false` |
| `.confetti-serial` | `HubView.countdowns.confetti_serial` |
| `.focus` / `.focus-serial` | `CountdownsView` chưa có; bridge giữ mặc định Slint |
| `select` / `open-detail` / `close-detail` / `remove` | Callback chọn mốc và mở/đóng chi tiết cập nhật state bridge; `remove(key)` gửi `UserInput::RemoveCountdown(id)` |
| `Countdown.fading` | Bridge giữ mốc đang xoá 400 ms để mờ dần; `CountdownItemView` chưa có trường fading |

Map: done — hub-mac #03 @ f2509e7

## Answer

Đã làm contract/countdowns.slint, screens/countdowns.slint, components/confetti.slint, fixture và 8 kịch bản demo. Flip chỉ chạy khi value đổi và mốc không đổi. Pháo giấy 30 mảnh từ Orb, tự gỡ sau 2 giây. Chưa có: ô bị xoá mờ dần (chỉ các ô còn lại dồn lên), hiệu ứng phóng lên khi chạm mốc ở danh sách (hoán đổi tức thì), đo fps trên Pi (ready-for-human).

## Comments

- 2026-11-23 (review hub-ui): Thêm `Countdown.fading`: xoá thì ô mờ dần 400 ms rồi mới gỡ. Flip chỉ chạy khi số giảm (`FlipText.num`). Chạm mốc ở danh sách: mốc chính phóng lên từ chỗ ô vừa chạm. Demo thêm `clear_all` cho Setting → Dữ liệu. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
