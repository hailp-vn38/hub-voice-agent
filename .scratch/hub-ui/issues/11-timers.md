# 11 — Màn Hẹn giờ

Status: resolved
Blocked by: 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Hẹn giờ" (báo hết giờ ở ticket 14, ô trên màn trung tâm ở ticket 07)

## Việc

- `screens/timers.slint` + `components/timer-ring.slint`.
- **Đầu màn:** "‹ Hẹn giờ · N đang chạy".
- **Thẻ hẹn giờ:** vòng tròn cạn dần, số còn lại lớn giữa vòng, nhãn, "X phút · xong lúc HH:MM", nút tạm dừng / +1 phút / huỷ.
- **Bố cục theo số lượng:** 1 → một vòng lớn giữa màn; 2 → hai cột; 3–4 → lưới 2×2.
- **Đặt nhanh** hàng dưới: 1 / 3 / 5 / 10 / 15 phút.
- **Trống:** vòng trống lớn + "Hey Jarvis, hẹn giờ 5 phút".
- **Hiệu ứng:**
  - Mới: `SpawnIn` từ Orb, nảy vào chỗ trống; thẻ khác dời mượt.
  - Vòng cạn liên tục: `animate` tuyến tính 1 s giữa hai giá trị `progress`; chữ số đổi mỗi giây.
  - `final-seconds` (10 s cuối): vòng và số màu ấm, vòng nhịp theo giây.
  - Tạm dừng: vòng mờ, số nhấp nháy chậm.
  - Huỷ/xoá: vòng co về tâm rồi biến mất (khi mục rời danh sách).
  - Điều chỉnh: vòng nở ra / co lại tới thời lượng mới.
  - Đã hết giờ: thẻ ghi "Đã hết giờ lúc HH:MM" cho tới khi được xoá.
- **Focus giọng nói:** thẻ `focus` sáng lên khi `focus-serial` đổi.
- `contract/timers.slint`:
  - `export enum TimerPhase { running, paused, final-seconds, finished }`.
  - `TimerCard { key, label, remaining, detail, progress: float, phase: TimerPhase }` (`remaining` "6:42", `detail` "7 phút · xong lúc 14:32" hoặc "Đã hết giờ lúc 14:32").
  - `global TimersData { in property <string> title; in property <[TimerCard]> timers; in property <string> focus; in property <int> focus-serial; callback quick-start(int); callback toggle-pause(string); callback add-minute(string); callback cancel(string); }`.

## Demo

Kịch bản: "Hẹn giờ luộc trứng 7 phút" (spawn), thêm tới 4 cái (đổi bố cục 1 → 2 → 2×2), "10 giây cuối", "Tạm dừng / tiếp tục", "Thêm 2 phút", "Huỷ trứng", "Hết giờ", "Trứng còn bao lâu?" (focus). Demo tự đếm ngược mỗi giây.

## Nghiệm thu

- [ ] Vòng cạn mượt, không giật theo giây; số đổi đúng mỗi giây.
- [ ] Ba bố cục đúng theo số hẹn giờ; chuyển bố cục mượt.
- [ ] Nút trên thẻ và nút đặt nhanh ≥ `Theme.touch-min`.
- [ ] Huỷ: co về tâm trước khi biến mất.

## Map với hub-core

Map: chờ hub-core #04 (hẹn giờ), #06 (hết giờ)

| Hợp đồng | UiState / UserInput |
|---|---|
| `TimersData.title` | |
| `.timers` | |
| `.focus` / `.focus-serial` | |
| `quick-start(minutes)` | |
| `toggle-pause(key)` / `add-minute(key)` / `cancel(key)` | |

## Answer

Màn Hẹn giờ xong: `contract/timers.slint`, `components/timer-ring.slint`, `components/timer-card.slint`, `screens/timers.slint`; gắn vào Shell. Demo mô phỏng (`demo/timers/`, logic thuần có test trong `sim_tests.rs`) tự đếm ngược mỗi giây, 8 kịch bản trong Demo panel. Nút Huỷ trên thẻ co về tâm 300 ms rồi gọi `cancel`; kịch bản "Huỷ trứng" xoá ngay (không co). Chưa kiểm bằng mắt trên màn hình.
