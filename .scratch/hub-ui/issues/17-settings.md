# 17 — Setting

Status: resolved
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Setting"

## Việc

- `screens/settings.slint` + một file mỗi danh mục trong `screens/settings/` (display, audio, voice, home, connection, data, system) để giữ file < 300 dòng.
- **Bố cục:** danh mục dọc bên trái (luôn hiện); bảng điều khiển giữa (cuộn dọc nếu dài); ô QR **Setup web** cố định bên phải: IP:cổng + mã QR + "Server, lịch, nhà, vị trí".
- **Chỉ chọn, không nhập chữ.** Áp dụng ngay, không có nút "Lưu". Thao tác xoá và tắt/khởi động lại → thẻ xác nhận (ticket 14).
- `components/slider.slint`, `components/choice.slint` (nhóm lựa chọn), `components/toggle.slint`. Cử chỉ bắt đầu trên slider thuộc slider, không thành vuốt điều hướng.
- **Danh mục:**
  - Hiển thị: độ sáng; tự chỉnh sáng (tắt / theo giờ); độ phân giải (chỉ mode được liệt kê) + thẻ "Giữ độ phân giải này? 10" đếm ngược; cỡ chữ (nhỏ / vừa / lớn → `Theme.text-scale`); vào Ambient sau (1 / 5 / 10 phút / không); Ambient ban đêm (bật/tắt, khung giờ chọn sẵn).
  - Âm thanh: âm lượng; âm giao diện (bật/tắt); chuông hẹn giờ và Reminder (chọn âm, chạm nghe thử).
  - Giọng nói: wake word (bật/tắt); độ nhạy (thấp / vừa / cao); tắt mic (chỉ bật lại bằng chạm); đọc Reminder bằng giọng server.
  - Nhà: chọn entity demo từ danh sách HA, tối đa 4 trên màn trung tâm.
  - Kết nối: trạng thái vela-server (đã kết nối / mất kết nối, độ trễ), mã Enrollment khi chưa đăng ký + nút Kết nối; Wi-Fi (tên, cường độ, IP); HA, Google Calendar, Open-Meteo (lần lấy gần nhất, lỗi gần nhất).
  - Dữ liệu: xoá toàn bộ ảnh; xoá cache thời tiết/lịch; xoá hẹn giờ / đếm ngược / Reminder (mỗi mục một nút).
  - Hệ thống: phiên bản; múi giờ (danh sách chọn); khởi động lại app; khởi động lại máy; tắt máy.
- **Màn voice settings hiện có:** OTA URL nhập ở Setup web theo concept; ô nhập `LineEdit` hiện tại giữ lại trong danh mục Kết nối cho tới khi hub-core #13 xong, ghi rõ trong Comments là ngoại lệ tạm.
- `contract/settings.slint`: `global SettingsData` gồm một struct mỗi danh mục (`DisplaySettings`, `AudioSettings`, `VoiceSettingsView`, `HomeSettings`, `ConnectionSettings`, `SystemSettings`), `setup-url: string`, `setup-qr: image`, `resolution-confirm: int` (giây còn lại, 0 = ẩn) và callback theo ý định: `set-brightness(float)`, `set-volume(float)`, `choose(string, string)` (khoá mục, giá trị), `toggle(string, bool)`, `preview-sound(string)`, `clear(string)`, `system(string)`, `keep-resolution()`, `connect-voice()`, `save-voice-url(string)`. Nội dung danh sách lựa chọn (mode màn hình, entity HA, âm chuông, múi giờ) đến từ hợp đồng.

## Demo

Kịch bản: đổi từng lựa chọn (demo áp lại vào fixture), "Đổi độ phân giải" (thẻ đếm 10 → tự huỷ), "Xoá toàn bộ ảnh" (thẻ xác nhận), "Chưa đăng ký voice" (mã Enrollment), "Tắt mic" (Orb xám có gạch).

## Nghiệm thu

- [ ] Kéo slider không chuyển màn; danh mục luôn hiện khi bảng cuộn.
- [ ] Không có ô nhập chữ ngoài ngoại lệ tạm OTA URL.
- [ ] Đổi cỡ chữ áp dụng ngay cho toàn app.
- [ ] Ô QR Setup web luôn hiện bên phải.
- [ ] Đường thật Kết nối voice (không demo) vẫn chạy như trước.

## Map với hub-core

Map: chờ hub-core #02 (Kết nối voice), #13 (Setup web URL); các danh mục khác chưa có ticket hub-core

| Hợp đồng | UiState / UserInput |
|---|---|
| `SettingsData.display` | |
| `.audio` | |
| `.voice` | |
| `.home` | |
| `.connection` | |
| `.system` | |
| `.setup-url` / `.setup-qr` | |
| `.resolution-confirm` / `keep-resolution()` | |
| `set-brightness` / `set-volume` | |
| `choose` / `toggle` / `preview-sound` | |
| `clear` / `system` | |
| `connect-voice()` / `save-voice-url(url)` | |

## Answer

Đã làm: `contract/settings.slint` (`SettingsData`), `screens/settings.slint` + 7 panel trong `screens/settings/`, components `slider`, `choice`, `toggle`, `setting-row`, `action-button`; Shell hiển thị Setting thay Placeholder; demo fixture `demo/settings.rs` và 3 kịch bản (Đổi độ phân giải, Chưa đăng ký voice, Tắt mic) + "Mở Setting".

- Thẻ xác nhận xoá/tắt máy là bản cục bộ trong `settings.slint` vì ticket 14 (`overlays/confirm.slint`) chưa có; thay bằng overlay chung khi 14 xong.
- Slider đặt `SettingsData.dragging` để Shell tắt vuốt phải khi đang kéo.
- Callback hệ thống tên `system-action` (tránh trùng property `system`).
- OTA URL: `LineEdit` tạm trong Kết nối (`SettingsData.ota-url`); đường thật (VoiceSettings, non-demo) giữ nguyên, chưa nối `SettingsData` vào bridge.
- Chưa có QR thật (`setup-qr` rỗng trong demo). Không có unit test (không có logic Rust ngoài demo).
