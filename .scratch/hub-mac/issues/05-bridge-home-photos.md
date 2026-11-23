# 05 — Bridge Nhà và Ảnh

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md) · Thiết kế: [../design.md](../design.md) §1.2.6 · UI: hub-ui #15, #16

## Việc

- `bridge/home.rs`: `HomeData` từ `HubView.home`; `toggle(key)` → `UserInput::ToggleHome(key)`.
- `bridge/images.rs`: cache `slint::Image` trên luồng UI (thread-local), khoá theo `PhotoView.key`.
  - Lưới: chỉ giải mã `thumbnail_path`.
  - `full`: chỉ cho `viewer_index` và hai ảnh kề; mục khác ảnh rỗng.
  - Xoá key không còn trong `HubView.photos`. Lỗi đọc file → ảnh rỗng, log.
- `bridge/photos.rs`: `PhotosData` từ `HubView.photos`.
  - `open(key)` → `Focus(Some(Focus::Photo(key)))`; `close-viewer` → `Focus(None)`.
  - `next`/`previous` → `Focus` của ảnh kề trong `viewer`, vòng như `hub_image_control`.
  - `toggle-pin(key)` → `PinPhoto { id, pinned: !pinned }`; `remove(key)` → `RemovePhoto { id }`.
  - `auto-advance` do bridge giữ (mặc định tắt).
- Điền bảng Map của hub-ui #15, #16.

## Nghiệm thu

- [ ] Test `images.rs` (phần không cần Slint): chọn đúng các key cần giải mã `full` theo `viewer_index` (đầu, giữa, cuối danh sách).
- [ ] Tay: màn Nhà có entity HA; bật/tắt đèn → `pending` rồi trạng thái mới.
- [ ] Tay: ảnh từ `hub_show_image` hiện trong lưới; mở viewer, tới/lui, đóng; ghim; xoá.
