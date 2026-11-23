# 07 — Màn trung tâm

Status: ready-for-human
Blocked by: 03, 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn hình" (bố cục màn chính), "Nguyên tắc", "Concept voice-first" mục 5; ô hẹn giờ ở "Màn Hẹn giờ"; ô mốc ở "Màn Đếm ngược"

## Việc

- `screens/center.slint` theo bố cục 16:9: thanh trạng thái; card đồng hồ + thời tiết (giờ và nhiệt độ là chữ lớn nhất, đọc được 1–2 m); lịch hôm nay; Reminder; đếm ngược; 2–4 thẻ nhà; hàng dưới dành Voice dock. Không tab, không cuộn.
- `contract/center.slint`:
  - `global Center { in property <string> time; in property <string> date; in property <CenterWeather> weather; in property <[CenterEvent]> today; in property <[CenterReminder]> reminders; in property <CenterTimer> timer; in property <CenterCountdown> countdown; in property <[HomeTile]> home; in property <CenterCard> focus; in property <int> focus-serial; callback open(Screen); callback toggle-home(string); }`.
  - `CenterWeather { icon, temp, summary, updated, stale: bool }`, `CenterEvent { key, time, title, now: bool }`, `CenterReminder { key, time, text }`, `CenterTimer { visible: bool, key, label, remaining, progress: float }`, `CenterCountdown { visible: bool, key, label, value, today: bool }`. `HomeTile` và `components/home-tile.slint` dùng chung với ticket 16 (`contract/home.slint`); ticket nào làm trước thì tạo, ticket sau dùng lại.
  - `export enum CenterCard { none, weather, calendar, reminders, countdown, home, timer }`.
- **Ô động:** có hẹn giờ chạy → ô nhỏ cho hẹn giờ sắp xong nhất (vòng + số); hết thì ô biến mất. Ngày đến mốc → ô mốc suốt ngày. Thêm/bớt ô dời layout mượt.
- **Phản ứng giọng nói:** `focus-serial` đổi → card `focus` phóng lớn tại chỗ trong lưới rồi trả về kích thước cũ.
- Chạm card → `Center.open(screen)` (đường tắt tới màn tính năng). Chạm thẻ nhà → `toggle-home(key)`.
- Dữ liệu cũ (offline): card ghi "cập nhật lúc HH:MM".

## Demo

Fixture đủ mọi card. Kịch bản: "Thêm hẹn giờ" (ô xuất hiện), "Hết hẹn giờ" (ô biến mất), "Hôm nay là Tết", "Hỏi mai có mưa" (card thời tiết phóng), "Offline".

## Nghiệm thu

- [ ] Ở 1920×1080, đọc được giờ và nhiệt độ từ xa; không có thanh cuộn.
- [ ] Ô hẹn giờ/mốc xuất hiện và biến mất không làm giật layout.
- [ ] Card phóng lớn rồi về đúng chỗ; chạm card mở đúng màn.
- [ ] Đổi `Theme.text-scale` lớn nhất vẫn không tràn.

## Map với hub-core

Map: done (hub-core #21; bridge ticket 06, commit `64295fb`)

| Hợp đồng | UiState / UserInput |
|---|---|
| `Center.time` / `.date` | `HubView.center.time/date` ← `UiState.now` (`present` #21) |
| `Center.weather` | `HubView.center.weather` ← `HubView.weather` (#18, #21) |
| `Center.today` | `HubView.center.today` ← calendar state (#10, #21) |
| `Center.reminders` | `HubView.center.reminders` ← `HubView.reminders` (#17, #21) |
| `Center.timer` | `HubView.center.timer` ← timer state / `HubView.timers` (#16, #21) |
| `Center.countdown` | `HubView.center.countdown` ← `HubView.countdowns` (#17, #21) |
| `Center.home` | `HubView.center.home` ← `HubView.home.tiles`, capped at four (#20, #21) |
| `Center.focus` / `.focus-serial` | `UiState.view.focus` / `UiState.events.focus` via `HubView.center` (#14, #21) |
| `Center.open(Screen)` | `UserInput::Open(Screen)` through the bridge's Screen mapping |
| `Center.toggle-home(key)` | `UserInput::ToggleHome(key)` |

## Answer

Đã làm `contract/center.slint`, `screens/center.slint`, `components/center-card.slint`, `components/center-timer.slint`, fixture `demo/fixtures/center.rs` và 5 kịch bản demo; Shell dùng `CenterScreen` thay Placeholder. `HomeTile`/`home-tile.slint` đã có từ ticket 16 nên dùng lại. Chưa kiểm tra bằng mắt ở 1920x1080 và `text-scale` 1.25 (chỉ build qua). `CenterTimer.progress` là phần còn lại 1 → 0.

## Comments

- 2026-11-23 (review hub-ui): Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
