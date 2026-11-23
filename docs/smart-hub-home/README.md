# Smart Home Hub — Responsive Slint Home

Đây là giao diện `Home` dựng bằng Slint, bám theo ảnh thiết kế minh hoạ và **có đầy đủ asset đi kèm**.

## Cấu trúc

```text
smart-hub-home/
├── home.slint                      # Toàn bộ màn Home, có breakpoint
├── assets/
│   ├── backgrounds/home-night.jpg # Ảnh nền ngôi nhà
│   ├── backgrounds/security-house.jpg # Ảnh nhà trong thẻ an toàn
│   ├── mascot/assistant.png       # Robot nền trong suốt
│   ├── icons/*.png                # Icon PNG để Slint sử dụng
│   ├── icons/svg/*.svg            # Icon SVG gốc để sửa lại
│   └── reference/design-reference.jpg
└── README.md
```

## Responsive Design

- **Ngang** (tỷ lệ rộng/cao ≥ 1,4): sidebar trái, lịch bên phải, thẻ thiết bị một hàng; kích thước theo canvas chuẩn 1280×600, scale **đồng nhất** để không méo hình.
- **Vuông hoặc dọc** (tỷ lệ < 1,4): đồng hồ, thời tiết và lịch xếp phía trên; thẻ thiết bị thành **lưới 3 cột × 2 hàng**; chuyển sidebar thành thanh điều hướng cuối màn hình. Canvas chuẩn 720×720.
- Hình nền luôn `image-fit: cover`. Khi tỷ lệ rất khác chuẩn, nội dung được căn giữa và giữ tỷ lệ (có thể còn khoảng trống nền để tránh méo hoặc cắt nút).

Phù hợp các màn hình hub 1280×600, 1024×600, 800×480 (bố cục ngang), và 720×720 (bố cục vuông). Ở màn nhỏ, Slint vẫn hiển thị nhưng bạn nên kiểm tra kích thước vùng chạm trên thiết bị thực.

## Cách dùng

Giữ nguyên `home.slint` và thư mục `assets` cùng cấp, rồi import `Home` trong dự án Slint hiện tại hoặc mở file bằng Slint Viewer.

```slint
import { Home } from "./home.slint";
```

Các property `clock-text`, `date-text`, `temperature-text`, `event-one`… có thể cập nhật từ backend. Các sự kiện tích hợp sẵn: `device-changed(string, bool)`, `navigation-requested(string)`, `voice-requested()`, `agenda-requested()`, `quick-action(string)`.

Đèn phòng ngủ mặc định bật để thể hiện trạng thái vàng/cam. Các nút đèn, quạt thay đổi trạng thái ngay trên UI và gọi callback để bạn nối MQTT/GPIO. Lưu ý: callback chưa triển khai điều khiển thiết bị thực; cần xử lý ở backend.

## Ghi chú

- Icon PNG có alpha và được đổi màu bằng property `colorize` của Slint, SVG là tài sản nguồn dùng để chỉnh sửa.
- Robot là PNG có alpha riêng, không dính vào nền, nên có thể thay đổi vị trí theo breakpoint.
- Không yêu cầu thêm thư viện icon hoặc tệp font. Font sử dụng font hệ thống (`Noto Sans` nếu có). Khi chạy trên thiết bị embedded, nên cài một font có đủ dấu tiếng Việt.
- Chưa kiểm thử được với trình biên dịch Slint ở môi trường đóng gói này; hãy kiểm tra cú pháp và renderer trên thiết bị đích trước khi triển khai production.
