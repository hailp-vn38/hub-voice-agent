# AI Hub — Hạn chế hiện tại

Cập nhật: 2026-11-24, theo nhánh `feat/hub-mac` và spec [hub-mac](../.scratch/hub-mac/spec.md).

Tài liệu này nói thẳng những gì hub **chưa** làm được, để người dùng và người phát triển không phải đoán. Ticket nào gỡ một hạn chế thì sửa tài liệu này trong cùng commit.

Mục 1 là tình trạng hôm nay trên nhánh. Mục 2 là những gì vẫn còn sau khi spec hub-mac xong (hub dùng cơ bản trên Mac). Mục 3 là những gì chỉ liên quan tới Pi.

## 1. Hôm nay trên `feat/hub-mac`

Ticket hub-mac 01–10 đã map xong: bridge ghi mọi màn từ hub-core, chuyển màn đi qua hub-core, chạm gửi `Activity`, có mic, loa, wake word và mức âm cho Orb. Các mục nghiệm thu **thủ công** trong từng ticket vẫn chờ người kiểm trên Mac.

| Hạn chế | Ảnh hưởng | Gỡ ở |
|---|---|---|
| Chưa kiểm tay: nói chuyện, chuông, wake word, Orb nhịp theo giọng, CPU lúc rảnh | Có thể còn lỗi chỉ lộ ra trên máy thật | Nghiệm thu tay hub-mac 01–10 |
| Không mở được thiết bị âm thanh thì app chạy **không tiếng** (log `Chạy không có âm thanh: …`) | Màn hình vẫn dùng được; không nói chuyện, chuông không kêu | Cắm loa/mic rồi mở lại app |

## 2. Còn lại sau spec hub-mac (Mac)

### Giọng nói và âm thanh

| Hạn chế | Ảnh hưởng | Cách xử lý tạm | Theo dõi |
|---|---|---|---|
| Mac không có khử vọng (AEC) | Dùng loa ngoài: server nghe lại TTS, tự ngắt lời; wake word có thể bắt nhầm | **Đeo tai nghe**, hoặc cắm XVF3800 qua USB ([development.md](development.md) "Âm thanh trên Mac") | ADR 0005 |
| Model wake word không có trong repo | Thiếu model thì không gọi "hey jarvis" được | Tải ba file ONNX vào `models/`; hoặc chạm Orb | hub-mac 09 |
| Model `hey_jarvis` là tiếng Anh, độ nhạy cố định ở mức vừa | Có thể bắt nhầm hoặc bỏ sót với giọng Việt | Nói rõ, gần mic | G5 (Setting giọng nói) |
| Không chỉnh âm lượng trong hub | — | Dùng phím âm lượng của Mac | G3, G4 |
| Không tắt mic được | Wake word luôn nghe | Thoát app | G5 |
| Hẹn giờ và Reminder dùng chung một tiếng chuông (`reminder.mp3`) | Không phân biệt được bằng tai | Nhìn lớp báo | Thêm file, sửa `audio/sounds.rs` |
| Sau wake word, nếu không nói gì thì hub vẫn ở trạng thái nghe | Mic mở cho tới khi server trả lời hoặc người dùng chạm Orb | Chạm Orb để kết thúc lượt | hub-core (`conversation`), chưa có ticket |
| Thời gian chờ nói tiếp 10 s tính từ lúc server báo TTS xong, không phải lúc loa phát xong | Câu trả lời dài có thể làm khoảng chờ thực tế ngắn hơn | Nói tiếp ngay | hub-core: dùng `TtsPlaybackFinished`, chưa có ticket |
| Chữ phản hồi không có nhấn (`Word.emphasis` luôn `false`) | Chỉ ảnh hưởng hiển thị | — | hub-core #15, chờ protocol |
| Tool phía server được nhận ra theo từ khoá trong tên (`calendar`, `weather`, `light`…) | Tool có tên lạ không mở màn nào | — | `view::screen_for_server_tool` |
| Nguồn và giấy phép của `ping.mp3`, `reminder.mp3` chưa ghi | Hai file đã nằm trong nhánh (commit `ba704ae`) nhưng chưa rõ được phép phân phối | Thay bằng file có giấy phép rõ trước khi phát hành | hub-mac 07 (`needs-info`) |

### Màn hình và tính năng

| Hạn chế | Ảnh hưởng | Theo dõi |
|---|---|---|
| Màn Setting chỉ có phần Kết nối voice; Hiển thị, Âm thanh, Giọng nói, Nhà, Dữ liệu, Hệ thống chưa hoạt động | Cấu hình qua Setup web (`http://<máy>:8080`) | G4–G8 |
| Chưa có tool `hub_open_screen`, `hub_show_card`, `hub_set_brightness`, `hub_set_volume` | Agent không mở màn tuỳ ý, không đưa thẻ nội dung, không chỉnh sáng/âm lượng | G1–G3 |
| Focus bằng giọng chưa làm sáng mục ở màn Đếm ngược và Reminder | Hỏi về một mốc hay Reminder mở đúng màn nhưng không đánh dấu mục; bridge để `focus` / `focus-serial` ở mặc định vì `CountdownsView` và `RemindersView` chưa cung cấp chúng | hub-core: thêm trường focus vào hai view; bridge hub-mac 03 đã ghi nhận phần còn thiếu |
| Thanh trạng thái: Wi-Fi luôn đầy, không có biểu tượng tắt mic, nền không đổi theo giờ | Chỉ là hiển thị | G9, G5, G11 |
| Ambient không có lịch ban đêm; luôn vào sau 5 phút | — | G4 |
| Màn trung tâm hiện 4 thiết bị Home Assistant đầu tiên, không chọn được | — | G6 |
| Lịch chỉ đọc Google Calendar; thời tiết chỉ từ Open-Meteo | — | Thiết kế (ADR 0007) |
| Setup web không xác thực trên mạng LAN | Ai cùng mạng cũng đổi được cấu hình | ADR 0009, quyết định có chủ đích |

## 3. Chỉ trên Pi (chưa làm, chưa kiểm)

Spec hub-mac để Pi lại sau. Các mục dưới đây **chưa được kiểm trên máy thật**:

- ReSpeaker XVF3800 làm cả đường vào và ra, AEC phần cứng, barge-in.
- Backend `linuxkms` + FemtoVG, fps của hiệu ứng Aurora trên GPU V3D, màn cảm ứng thật.
- `libopus` và ONNX Runtime (`ort`) cho aarch64 trong image Docker bookworm (ADR 0011).
- Lần đầu khởi động khi chưa có mạng: điểm truy cập Wi-Fi và hai mã QR (G10). Hiện phải cắm Ethernet hoặc cấu hình Wi-Fi bằng tay.
- Độ sáng màn hình, khởi động lại / tắt máy qua `System` (G4, G8).
- Nhiệt độ và CPU khi chạy lâu.

Danh sách đầy đủ việc chỉ kiểm được trên Pi: [development.md §6](development.md).
