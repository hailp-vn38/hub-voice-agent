# Phần hub-core còn thiếu để map UI

Cập nhật: 2026-11-23. Spec: [spec.md](spec.md).

UI làm trước bằng dữ liệu demo theo các ticket trong `issues/`. Một số hợp đồng chưa có ticket hub-core nào cung cấp dữ liệu hoặc nhận thao tác, nên **chưa map được**. Tài liệu này liệt kê các chỗ đó: hợp đồng nào, hub-core cần làm gì, căn cứ ở đâu. Khi tạo ticket hub-core cho một mục, ghi số ticket vào cột "Ticket hub-core" và vào bảng Map của [spec.md](spec.md).

## Tóm tắt

| # | Phần thiếu | Hợp đồng UI chờ | Ticket UI | Căn cứ | Ticket hub-core |
|---|---|---|---|---|---|
| G1 | Tool `hub_open_screen` | `Nav.screen` (đổi màn theo tool) | 03 | concept.md "Agent điều khiển hub" | chưa có |
| G2 | Tool `hub_show_card` | `FloatingCards`, `Spawn` | 06 | concept.md "Concept voice-first" mục 4 | chưa có |
| G3 | Tool `hub_set_brightness` / `hub_set_volume` | `VoiceSlider`, `SettingsData.display/audio` | 14, 17 | concept.md "View theo giọng nói" (dòng Setting) | chưa có |
| G4 | Setting Hiển thị + Âm thanh | `SettingsData.display`, `.audio`, `Theme.text-scale` | 02, 17 | concept.md "Setting" | chưa có |
| G5 | Setting Giọng nói | `SettingsData.voice`, `Voice.mic-muted`, `StatusBar.mic-muted` | 03, 04, 17 | concept.md "Setting"; ADR 0005 | chưa có |
| G6 | Setting Nhà (chọn entity) | `SettingsData.home`, `Center.home` | 07, 17 | concept.md "Setting", §5 HA | chưa có (gần #11) |
| G7 | Setting Kết nối (ngoài voice) | `SettingsData.connection` | 17 | concept.md "Setting" | một phần ở #02 |
| G8 | Setting Dữ liệu + Hệ thống | `SettingsData.system`, callback `clear`, `system` | 17 | concept.md "Setting" | chưa có |
| G9 | Trạng thái mạng qua `System` | `StatusBar.wifi-strength`, `SettingsData.connection` | 03, 17 | ADR 0010 | chưa có |
| G10 | Lần đầu khởi động: điểm truy cập + hai QR | `FirstBootData` | 19 | ADR 0010; concept.md "Lần đầu khởi động" | chưa có |
| G11 | Nền theo giờ trong ngày | `StatusBar.day-phase` | 03 | concept.md "Hiệu ứng theo trạng thái" (Idle) | chưa có (nhỏ, gần #05) |

## Chi tiết

### G1 — `hub_open_screen`

- Tool trong danh sách đã chốt nhưng không ticket nào có: #05 chỉ có tool của tính năng mở màn và `ServerTool`.
- Cần: tham số màn (thời tiết, lịch, nhà, hẹn giờ, đếm ngược, Reminder, ảnh, setting, trung tâm) → `view.open(screen, None)`; đánh thức Ambient trước khi chuyển; reply OK.
- Gần nhất: ticket #05.

### G2 — `hub_show_card`

- Thẻ nội dung do agent soạn. Thẻ thuộc tính năng → mở màn đó và bay vào chỗ của nó; thẻ chung → nổi trên Voice dock, mờ đi khi hội thoại kết thúc.
- Cần: tham số (loại, tiêu đề, nội dung, tính năng nếu có), danh sách thẻ nổi trong `UiState`, cờ `fading` theo `conversation`, serial spawn.
- Câu hỏi mở: thẻ "thuộc tính năng" (sự kiện, Reminder) là chỉ hiển thị hay tạo mục thật? concept.md chưa nói; cần chốt trước khi làm.

### G3 — `hub_set_brightness` / `hub_set_volume`

- Cần: tool nhận giá trị (0–100), ghi config, gửi `System` (độ sáng) hoặc `AudioCommand::SetVolume`, phát serial cho `VoiceSlider`. Không đổi màn.
- Phụ thuộc G4 (cùng giá trị độ sáng/âm lượng với Setting).

### G4 — Setting Hiển thị + Âm thanh

- Hiển thị: độ sáng, tự chỉnh sáng (tắt / theo giờ), độ phân giải (danh sách mode; áp mode là câu hỏi mở codebase.md §7), cỡ chữ (nhỏ / vừa / lớn), vào Ambient sau (1 / 5 / 10 phút / không), Ambient ban đêm (bật/tắt, khung giờ).
- Âm thanh: âm lượng, âm giao diện bật/tắt, chọn chuông hẹn giờ và Reminder, nghe thử.
- Cần: field trong `ConfigStore`, `UserInput` cho từng lựa chọn, áp dụng ngay (độ sáng qua `System`, Ambient qua `view`, âm qua `AudioCommand`).
- Port `System` cần thêm: `set_brightness`, `display_modes`.

### G5 — Setting Giọng nói

- Wake word bật/tắt + độ nhạy → `AudioCommand::SetWakeWord` (đã có trong `audio.rs`).
- **Tắt mic:** wake word ngừng hẳn, Orb xám có gạch, chỉ bật lại bằng chạm; không có tool tắt mic (ADR 0008). Cần trạng thái `mic_muted` trong `conversation` hoặc config và chặn `WakeWord` khi tắt.
- Đọc Reminder bằng giọng server bật/tắt: #08 đã nhắc "setting đọc bằng giọng bật" nhưng không ticket nào tạo setting đó.

### G6 — Setting Nhà

- Chọn entity demo từ danh sách HA, tối đa 4 trên màn trung tâm. #11 có entity hiển thị nhưng không có bước chọn và lưu lựa chọn.

### G7 — Setting Kết nối

- #02 lo trạng thái vela-server và mã Enrollment. Còn thiếu: độ trễ vela-server, Wi-Fi (tên, cường độ, IP — xem G9), HA / Google Calendar / Open-Meteo (lần lấy gần nhất, lỗi gần nhất). Từng nguồn có ticket riêng (#09, #10, #11) nhưng không ticket nào gom trạng thái này vào `UiState`.

### G8 — Setting Dữ liệu + Hệ thống

- Dữ liệu: xoá toàn bộ ảnh, xoá cache thời tiết/lịch, xoá hẹn giờ / đếm ngược / Reminder; có hỏi lại (UI lo thẻ xác nhận).
- Hệ thống: phiên bản app, múi giờ (mặc định Asia/Ho_Chi_Minh; ảnh hưởng `Clock`), khởi động lại app / máy, tắt máy (qua `System`).

### G9 — Trạng thái mạng

- Port `System` mới chỉ có `time_synced()` (#06). Cần thêm trạng thái mạng (Wi-Fi tên, cường độ, IP, có Ethernet) để hiện ở thanh trạng thái và Setting, và để G10 quyết định khi nào vào chế độ lần đầu.

### G10 — Lần đầu khởi động

- ADR 0010 đã chốt; chưa có ticket. Cần: phát hiện không có Wi-Fi/Ethernet, bật điểm truy cập qua `System` (NetworkManager trên Pi, giả trên Mac), sinh nội dung hai QR (chuỗi Wi-Fi của AP, URL Setup web), nhận Wi-Fi nhà từ Setup web (#13), chuyển mạng, báo IP mới hoặc lỗi.
- Phụ thuộc #13 (Setup web) và G9.

### G11 — Nền theo giờ trong ngày

- UI cần một số 0..1 theo giờ địa phương để đổi màu nền khi Idle, cập nhật mỗi phút. Nhỏ; có thể làm cùng #05 hoặc khi map `StatusBar`.

## Đã có ticket (2026-11-23)

Lớp trình bày và phần thiếu không thuộc G1–G11 đã thành ticket hub-core 14–21: `present` + `now` + serial (#14), chữ phản hồi / ToolTrail / chạm Orb (#15), thời lượng hẹn giờ + thao tác chạm hẹn giờ (#16), đếm ngược + Reminder (#17), WeatherView có kiểu (#18), lịch (#19), nhà + ảnh (#20), trung tâm + Menu (#21). G1–G11 vẫn chưa có ticket.

## Không phải phần thiếu

Các mục sau có ticket hub-core, chỉ chờ ticket đó xong: điều hướng và Ambient (#05), voice và chữ phản hồi (#03), hẹn giờ (#04), báo (#06), mốc (#07), Reminder (#08), thời tiết (#09), lịch (#10), nhà (#11), ảnh (#12), Setup web URL (#13).
