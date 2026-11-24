# AI Hub

Thiết bị hub AI cá nhân trên Raspberry Pi 4 có màn cảm ứng: là Physical Device client của vela-server và là hub thông tin trong nhà. Tên trong ngoặc là tên dùng trong code.

## Thiết bị và server

**Hub** (`hub`):
Thiết bị Pi 4 + màn hình + mic array chạy app này.
_Avoid_: kiosk, máy, client

**vela-server**:
Server voice agent mà Hub kết nối qua Voice Protocol.
_Avoid_: backend, server AI

**Physical Device**:
Loại client phần cứng của vela-server; Hub là một Physical Device.

**Enrollment** (`enrollment`):
Bước đăng ký Hub với vela-server lần đầu bằng mã kích hoạt hiện trên màn hình.
_Avoid_: pairing, đăng nhập

## Voice

**Hội thoại** (`conversation`):
Khoảng từ lúc wake word hoặc chạm Orb tới khi chạm Orb lần nữa hoặc đang nghe mà im lặng 5 phút; trong đó mic được stream lên server.
_Avoid_: session (trùng Voice Session phía server)

**Lượt nói** (`turn`):
Một câu của người dùng và câu trả lời của agent trong một Hội thoại.

**Wake word** (`wake_word`):
Từ khoá `hey_jarvis` nghe cục bộ để bắt đầu Hội thoại.
_Avoid_: hotword, keyword

**Direct Speech** (`direct_speech`):
Văn bản Hub gửi để server đọc thẳng bằng TTS, không qua ASR/LLM; dùng để đọc Reminder.

## Tool

**Device MCP tool** (`tool`):
Tool `hub_*` chạy trên Hub, agent gọi qua kết nối voice; Hub nhận đủ tham số.
_Avoid_: command, action

**Tool phía server** (`server_tool`):
Tool chạy trên vela-server; Hub chỉ biết tên và trạng thái (`started` / `completed` / `failed`).

**Focus** (`focus`):
Phần của màn được làm nổi bật theo câu hỏi (ngày, giờ, nhãn).
_Avoid_: highlight, selection

## Giao diện

**Màn trung tâm** (`Screen::Center`):
Màn mặc định với các card hub; không tab, không cuộn. Nhãn "Home" trên Thanh điều hướng.
_Avoid_: home screen (trùng tính năng Nhà), dashboard

**Menu tính năng** (`Screen::Menu`):
Lưới 4 × 2 ô mở các Màn tính năng; nhãn "Ứng dụng" trên Thanh điều hướng, cũng vào được bằng vuốt phải → trái từ Màn trung tâm.
_Avoid_: thiết bị, app drawer

**Thanh điều hướng** (`nav_rail`):
Cột nút cố định ở mép trái màn: Home, Ứng dụng, Thống kê, Cài đặt.
_Avoid_: sidebar, tab bar

**Màn tính năng** (`Screen::Feature`):
Màn của một tính năng (Thời tiết, Lịch, Nhà, Hẹn giờ, Đếm ngược, Reminder, Ảnh, Setting).
_Avoid_: trang, tab, màn con

**Ambient** (`Screen::Ambient`):
Màn đồng hồ lớn, độ sáng thấp khi không có tương tác hoặc ban đêm.
_Avoid_: screensaver, sleep

**Voice dock** (`voice_dock`):
Thanh cố định trải ngang đáy mọi màn, chứa Orb ở đầu trái, Chữ phản hồi và Dải bước tool.
_Avoid_: thanh voice, màn voice

**Orb** (`orb`):
Nút sống ở đầu trái Voice dock (icon mic và vạch sóng) đại diện cho agent; mọi thứ agent tạo ra đều sinh ra từ Orb.
_Avoid_: avatar

**Chữ phản hồi** (`response_text`):
Dòng chữ chạy từ Orb sang phải, hiện lời agent và transcript người dùng.
_Avoid_: subtitle, caption, kinetic text

**Dải bước tool** (`tool_trail`):
Dải chấm cạnh Orb, mỗi chấm một tool được gọi trong Lượt nói.

**Trạng thái báo** (`alert`):
Lớp toàn màn khi Hẹn giờ hết giờ hoặc Reminder đến hạn.
_Avoid_: notification, popup

**Tạm xem** (`preview_location`):
Thời tiết của một thành phố khác hỏi bằng giọng nói, không thay Vị trí nhà.

**Vị trí nhà** (`home_location`):
Toạ độ thời tiết mặc định, đặt qua Setup web.

**Setup web** (`setup_web`):
Trang web Hub tự phục vụ trong LAN để nhập setting cần gõ chữ.
_Avoid_: admin page, web config

## Tính năng cục bộ

**Hẹn giờ** (`Timer`):
Đếm lùi một khoảng thời gian (vd. 7 phút), có nhãn; tối đa 4.
_Avoid_: alarm, báo thức

**Đếm ngược** (`Countdown`):
Đếm số ngày tới một Mốc.
_Avoid_: timer

**Mốc** (`Countdown` target):
Ngày/giờ đích của Đếm ngược, có thể lặp hằng năm theo dương hoặc âm lịch.

**Reminder** (`Reminder`):
Lời nhắc tại một thời điểm, có thể lặp hằng ngày/tuần; lưu trên Hub, không phải Reminder của vela-server.
_Avoid_: lời nhắc server, notification

**Đã lỡ** (`missed`):
Reminder đến hạn lúc Hub tắt hoặc chưa chạy.

**Ghim** (`pinned`):
Ảnh được giữ lại, không bị xoá khi cache ảnh đầy.
