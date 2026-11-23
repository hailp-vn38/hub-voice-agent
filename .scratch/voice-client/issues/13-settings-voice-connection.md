# 13 — Settings: Kết nối voice và hiển thị mã kích hoạt

Status: ready-for-human
Blocked by: 03
Spec: [../spec.md](../spec.md)

## Việc

Thêm mục **Kết nối voice** trong Settings của hub (`apps/hub`, Slint) để người dùng thiết lập kết nối tới vela-server và thấy mã kích hoạt.

- Ô nhập **OTA URL** (ví dụ `http://<server>/voice/ota`). Lưu → cập nhật `watch::Sender<VoiceConfig>` → voice-client kết nối lại (bất biến 6). URL sai định dạng → báo lỗi ngay ở form, không gửi config.
- **Device ID** và **Client ID** sinh cùng lúc (UUID) lần đầu chạy. Hiện Device ID (chỉ đọc) để người dùng đối chiếu trên admin web. Device ID sinh một lần và giữ nguyên qua các lần khởi động, vì server nhận diện Device bằng nó.
- Hiện **trạng thái** theo `Connection(..)`: Đang kết nối / Đã kết nối / Chờ kích hoạt / Offline (thử lại sau N s). Chưa lưu OTA URL thì voice-client chưa chạy và hiện "Chưa cấu hình".
- Khi `Connection(Enrolling)`: hiện **mã kích hoạt** từ `Activation { code }`, chữ lớn, kèm hướng dẫn "Nhập mã này trong mục Thêm thiết bị trên admin web, rồi bấm Kết nối", và nút **Kết nối** → gửi `VoiceCommand::Connect`. Nút chỉ hiện khi `Enrolling`. `Ready` → ẩn mã và nút.
- OTA URL, Device ID và Client ID lưu bền trên máy hub, đọc lại khi khởi động để `spawn` voice-client.

## Nghiệm thu

- [x] Nhập OTA URL hợp lệ, lưu → voice-client nhận config mới; khởi động lại hub vẫn giữ URL, Device ID và Client ID (kiểm thử qua controller và config store).
- [x] URL không hợp lệ → báo lỗi, config không đổi (kiểm thử qua controller và config store).
- [x] Server giả trả `activation` → Settings hiện đúng mã và trạng thái "Chờ kích hoạt"; bấm Kết nối lần nữa, OTA trả `websocket` → trạng thái "Đã kết nối", mã biến mất.

## Comments

Cần chốt khi triage:

- Nơi lưu config (file TOML/JSON trong thư mục dữ liệu app?) và module nào sở hữu (`hub-core` hay `apps/hub`). Spec voice-client đang để "Lưu `VoiceConfig` và Setup web" ngoài phạm vi crate.

- Quyết định triển khai: `apps/hub` sở hữu voice settings, lưu `[voice]` trong `<data>/config.toml` quyền 0600 theo ADR 0012. Thư mục lấy từ `AI_HUB_DATA_DIR`, `STATE_DIRECTORY` hoặc `.data`; chưa có URL thì chưa chạy voice-client. Các test controller với server OTA/WS giả xác nhận mã, `Connect` và `Ready`; nghiệm thu trực quan trên cửa sổ Slint và với vela-server thật còn chờ người kiểm thử.

- Nghiệm thu 3 được kiểm qua controller và `SettingsState::status_label` (nhãn hiện trên màn hình). Nhìn trực tiếp cửa sổ Slint và chạy với vela-server thật vẫn chờ người kiểm thử.
