# 15 — Màn Ảnh và xem ảnh

Status: ready-for-human
Blocked by: 06
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Ảnh"

## Việc

- `screens/photos.slint` (lưới) + `screens/photo-viewer.slint` (xem toàn màn; một phần của màn Ảnh, không phải màn riêng trong `Screen`).
- **Lưới:** "‹ Ảnh · N ảnh"; "M đã giữ"; 4 × 2 thumbnail mới nhất trước, mỗi ô có chú thích + thời điểm; cuộn **dọc** xem cũ hơn; ô lỗi: icon ảnh hỏng + "Không tải được"; trống: "Hey Jarvis, cho tôi xem ảnh vịnh Hạ Long".
- **Xem ảnh:**
  - Toàn màn, Ken Burns (zoom/trượt rất chậm); chú thích + tên miền nguồn góc trái dưới; Voice dock vẫn ở góc phải dưới.
  - Nhiều ảnh (≤ 6): chấm chỉ số mép trên; tự chuyển mỗi ~6 giây (Slint hẹn giờ trình bày); **chạm mép phải/trái** → `next()` / `previous()`. Không vuốt ngang để chuyển ảnh.
  - Chạm giữa ảnh → hiện/ẩn chú thích. Vuốt trái → phải → về lưới (`close-viewer()`).
  - Icon ghim trên ảnh → `toggle-pin(key)`; ghim "nhảy lên" khi đổi trạng thái.
  - Thẻ chi tiết ảnh có nút Xoá → thẻ xác nhận (ticket 14) → `remove(key)`.
- **Hiệu ứng tải:** thẻ khung chờ `SpawnIn` từ Orb, nở dần thành toàn màn khi ảnh xong (crossfade); lỗi → thẻ rung nhẹ, "Không tải được ảnh".
- `contract/photos.slint`:
  - `export enum PhotoState { loading, ready, failed }`.
  - `Photo { key, thumb: image, full: image, caption, source, time, pinned: bool, state: PhotoState }`.
  - `global PhotosData { in property <string> title; in property <string> pinned-text; in property <[Photo]> photos; in property <bool> viewing; in property <[Photo]> viewer; in property <int> viewer-index; in property <bool> auto-advance; callback open(string); callback next(); callback previous(); callback close-viewer(); callback toggle-pin(string); callback remove(string); }`.
  - `auto-advance` true cho ảnh agent gửi; ảnh mở từ lưới thì không tự chuyển.

## Demo

Fixture: 10 ảnh mẫu trong `assets/demo/` (chỉ dùng cho demo, ảnh tự tạo hoặc giấy phép tự do), một ô lỗi. Kịch bản: "Agent gửi 3 ảnh Đà Lạt" (khung chờ → toàn màn, tự chuyển), "Ảnh lỗi", "Ảnh tiếp / trước", "Giữ ảnh này", "Cho xem lại ảnh lúc nãy".

## Nghiệm thu

- [ ] Chạm mép trái/phải đổi ảnh; vuốt ngang không đổi ảnh mà về lưới.
- [ ] Ken Burns mượt, không giật khi đổi ảnh (crossfade).
- [ ] Lưới cuộn dọc được; không cuộn ngang.
- [ ] Đo fps Ken Burns ảnh 1080p trên Pi (ready-for-human).

## Map với hub-core

Map: done · hub-core #12; hub-mac #05, commit `9c100af`

| Hợp đồng | UiState / UserInput |
|---|---|
| `PhotosData.title` / `.pinned-text` | `HubView.photos.title` / `HubView.photos.pinned_text` |
| `.photos` | `HubView.photos.photos`; `thumbnail_path` tạo ảnh lưới, `image_path` tạo ảnh full theo vùng xem |
| `.viewing` / `.viewer` / `.viewer-index` / `.auto-advance` | `HubView.photos.viewing` / `.viewer` / `.viewer_index`; `auto-advance` là trạng thái bridge, mặc định tắt và bật khi ảnh agent tải xong |
| `open` / `next` / `previous` / `close-viewer` | `Focus(Some(Focus::Photo(key)))` / `Focus(None)`; next/previous đi vòng theo `viewer` |
| `toggle-pin` / `remove` | `PinPhoto { id, pinned: !pinned }` / `RemovePhoto { id }` |

## Answer

Implemented `contract/photos.slint`, `screens/photos.slint` (grid), `screens/photo-viewer.slint` (Ken Burns, crossfade, edge taps, auto-advance timer, swipe right to grid, pin bounce), demo adapter `src/ui/demo/photos.rs` with 10 generated images in `assets/demo/` and 6 scenarios. Deviations: Xoá calls `remove` directly (confirm card from ticket 14 not yet wired); Shell right-swipe is disabled while the viewer is open so the viewer handles it; pin icon added (`pin.svg`). fps on Pi not measured.

## Comments

- 2026-11-23 (review hub-ui): Nút Xoá trong trình xem ảnh mở thẻ xác nhận dùng chung (`overlays/confirm.slint`) trước khi gọi `remove`. Demo thêm `clear_all` cho Setting → Dữ liệu. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
