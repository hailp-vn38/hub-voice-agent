---
status: accepted
---

# Một process, một actor giữ trạng thái; UI chỉ đọc UiState

Hub chạy trong một process: Slint event loop trên main thread, tokio runtime trên thread khác, audio trên thread riêng. Mọi trạng thái của hub do một actor trong `hub-core` giữ; UI nhận ảnh chụp `UiState` qua `watch` channel và gửi `UserInput` ngược lại. Slint chỉ làm hiển thị và animation (kể cả kéo theo ngón tay khi vuốt), không chứa luật nghiệp vụ.

Chọn vậy để giọng nói, chạm và sự kiện theo giờ (hết giờ, Reminder) cùng đi qua một chỗ, không có hai nơi cùng sửa một trạng thái, và test được hành vi mà không cần UI.

## Consequences

- Hiệu ứng một lần (thẻ sinh ra từ Orb, từ mới của chữ phản hồi) mang trong `UiState` kèm id tăng dần; UI chạy animation khi thấy id mới.
- Dữ liệu tần số cao (biên độ mic, năng lượng TTS) đi kênh `watch` riêng từ audio thẳng tới UI, không qua actor.
