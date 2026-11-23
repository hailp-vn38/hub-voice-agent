# 12 — Ảnh

Status: completed
Blocked by: 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `ImageFetcher`; adapter `sources/image_fetch.rs`: giới hạn kích thước, chặn LAN/localhost, thu nhỏ + thumbnail.
- `src/features/photos/` (khuôn ticket 04): chỉ mục ảnh, ghim, ≤ 100 ảnh / 200 MB, xoá ảnh cũ nhất chưa ghim; `features/photos/store.rs` + thư mục `<data>/photos/`.
- Tool hiển thị ảnh từ URL.

## Nghiệm thu

- [x] Adapter từ chối URL `127.0.0.1`, `192.168.x.x`, ảnh quá lớn.
- [x] Unit test chính sách xoá: vượt giới hạn → xoá cũ nhất không ghim.
- [x] Test qua `run()`: tool show_image → màn Ảnh hiện ảnh mới sau khi fake trả về.

## Tiến độ

- Adapter `HttpImageFetcher` đã có giới hạn tải, kiểm tra địa chỉ DNS cho từng bước chuyển hướng, và tạo ảnh/thumbnail PNG đã thu nhỏ.
- Đã tích hợp `Ports`, cache, ghim và tool; test `run()` với fake và chính sách xoá đã pass.
- Test transport nội bộ xác minh từ chối `Content-Length` và body streaming vượt 10 MiB, cùng redirect đến localhost; production vẫn kiểm tra DNS ở mỗi bước.
