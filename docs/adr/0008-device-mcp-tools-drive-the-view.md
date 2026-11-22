---
status: accepted
---

# Device MCP: mỗi tool một việc, tool điều khiển view

Hub khai báo `features.mcp=true` và cung cấp các tool `hub_*` (danh sách trong concept.md), mỗi tool một việc. Gọi một tool gắn với tính năng — đọc hay ghi — thì hub chuyển tới màn của tính năng đó và làm nổi bật phần liên quan theo tham số `focus`. Tool đọc trả **đúng dữ liệu đang hiển thị**, để số agent nói khớp với màn hình.

Với tool phía server (hub chỉ nhận `name` + `state` qua `features.tool_status=true`), hub dùng bảng ánh xạ tên tool → màn và hiển thị dữ liệu từ cache của mình.

## Consequences

- `hub_show_image` nhận URL (1–6), không nhận ảnh base64 qua WebSocket; hub tự tải với giới hạn kích thước, loại file, timeout và thu nhỏ trước khi hiển thị.
- Tool ghi theo nhãn (huỷ hẹn giờ, xoá đếm ngược…) trả lỗi "mơ hồ" kèm ứng viên khi khớp nhiều mục, để agent hỏi lại.
- Không có tool tắt mic: tắt bằng giọng rồi thì không bật lại bằng giọng được.
