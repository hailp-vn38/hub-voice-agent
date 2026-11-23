# AI Hub — Phát triển trên máy Mac

> Trạng thái: **kế hoạch**, repo chưa có code. Lệnh và đường dẫn dưới đây là cách dự kiến tổ chức; cập nhật lại khi dựng workspace. Cập nhật: 2026-11-22.

Xem thiết kế tổng thể ở [concept.md](concept.md).

## 1. Máy phát triển

| Thành phần | Giá trị |
|---|---|
| Máy | Mac Apple Silicon (arm64), 16 GB RAM |
| Rust | 1.98 (rustup), target `aarch64-apple-darwin` |
| Docker | 28.x — dùng để build bản cho Pi |
| Thiết bị đích | Raspberry Pi 4, Raspberry Pi OS Lite (Debian bookworm, arm64) |

Ý chính: **viết và chạy hub ngay trên Mac**, build bản Pi trong container `linux/arm64` (chạy native trên Apple Silicon, không giả lập), rồi chép binary sang Pi.

## 2. Cấu trúc repo dự kiến

```
ai-hub/
├── Cargo.toml              # workspace
├── crates/voice-client/    # kết nối Voice Protocol, không UI, không thiết bị âm thanh
├── crates/hub-core/        # hành vi của hub, test bằng `cargo test -p hub-core`
├── apps/hub/               # app Slint chạy trên Mac và Pi
│   └── ui/                 # file .slint
├── docker/pi-build.Dockerfile
├── deploy/ai-hub.service   # systemd cho Pi
└── docs/
```

## 3. Chạy trên Mac

Những gì app thật trên Mac chưa làm được: [limitations.md](limitations.md). Kế hoạch hoàn thiện bản Mac: [.scratch/hub-mac/spec.md](../.scratch/hub-mac/spec.md).

### Backend Slint theo nơi chạy

| Nơi chạy | Feature Slint | Ghi chú |
|---|---|---|
| Mac (phát triển) | `backend-winit` | Cửa sổ desktop; chuột thay ngón tay |
| Pi 4 | `backend-linuxkms` + `renderer-femtovg` | Vẽ thẳng ra DRM/KMS, không X11/Wayland |

Chọn bằng feature của crate `apps/hub` (dự kiến `desktop` mặc định, `pi` khi build cho Pi). Code ngoài phần chọn backend giống nhau trên hai nơi.

### Chế độ demo (UI không cần hub-core)

`scripts/run.sh run --features demo` mở app với dữ liệu fixture và cửa sổ Demo panel (nút kịch bản); không chạy hub-core hay voice-client. Bản Pi không biên dịch `src/ui/demo/`.

### Mô phỏng màn hình 5 inch 1080p

- Cửa sổ kích thước logic 1920×1080, thu nhỏ bằng `SLINT_SCALE_FACTOR` (vd. `0.5`) cho vừa màn Mac.
- Vuốt menu: nhấn giữ chuột và kéo ngang. Chạm: click.
- Kiểm tra cỡ chữ và vùng chạm theo đơn vị logic, không theo pixel Mac.
- Dùng live-preview của Slint (extension VS Code hoặc `slint-viewer`) khi chỉnh file `.slint`.

### vela-server chạy cục bộ

- Chạy vela-server trên cùng Mac bằng compose của repo `AI_Agent_Voice_rust`.
- Hub dùng OTA URL `http://localhost:<port>/voice/ota` để lấy địa chỉ WebSocket và token; URL được đặt trong **Settings → Kết nối voice** (xem bên dưới).
- Cấu hình vela-server với `devices.enrollment.transport = "ota"`. Chế độ `"websocket"` không được hub hỗ trợ vì không phân biệt được Enrollment Session với Voice Session.
- Mở **Settings → Kết nối voice**, nhập OTA URL rồi lưu. Device ID, Client ID và URL được lưu trong `[voice]` của `<data>/config.toml`; thư mục dữ liệu lấy từ `AI_HUB_DATA_DIR`, `STATE_DIRECTORY` hoặc `.data` theo thứ tự ưu tiên. Khi hiện mã kích hoạt, nhập mã trên admin web rồi bấm **Kết nối**.
- Đăng ký hub như một Physical Device qua Enrollment, như thiết bị thật.
- Server cần bật `barge_in.enabled` và `barge_in.trust_client_aec_feature` khi test barge-in; client gửi `features.mcp`, `features.tool_status`, `features.direct_tts`.

### Âm thanh trên Mac

| Cách | Khi nào |
|---|---|
| Mic/loa có sẵn của Mac (`cpal` → CoreAudio) | Phát triển UI và luồng hội thoại; **đeo tai nghe** để loa không lọt vào mic (Mac không có AEC cho app) |
| ReSpeaker XVF3800 cắm USB vào Mac | Test wake word, AEC và barge-in thật. Chọn XVF3800 làm **cả ngõ vào và ngõ ra**, loa cắm vào cổng ra của mic array |

- Lần đầu dùng mic, macOS hỏi quyền Microphone cho terminal/IDE đang chạy app.
- Wake word `hey_jarvis` (openWakeWord qua `ort`) chạy được trên macOS arm64; model ONNX tải về thư mục `models/` (không commit).

### Dịch vụ bên ngoài

Open-Meteo, Google Calendar API, Home Assistant và Setup web chạy trên Mac giống hệt Pi.

- Google OAuth: khi dev trên Mac, trình duyệt chạy cùng máy nên redirect `http://127.0.0.1:<port>` về thẳng app — không cần bước chép URL như trên Pi.
- Home Assistant: trỏ tới HA trong mạng nhà, hoặc chạy HA bằng Docker để demo.

## 4. Build bản cho Pi 4

Build trong container Debian bookworm `linux/arm64` để glibc khớp Raspberry Pi OS.

Thư viện hệ thống cần trong image (dự kiến): `libudev-dev`, `libinput-dev`, `libdrm-dev`, `libgbm-dev`, `libegl-dev`, `libgles-dev`, `libxkbcommon-dev`, `libfontconfig-dev`, `libasound2-dev`, `pkg-config`. ONNX Runtime cho `ort` cần bản `aarch64-linux`.

Luồng dự kiến:

1. Dựng image build một lần (`docker/pi-build.Dockerfile`, `--platform linux/arm64`).
2. Chạy `cargo build --release -p hub --no-default-features --features pi` trong container, mount repo và cache `~/.cargo/registry`, thư mục `target` riêng (vd. `target/pi`) để không lẫn với bản Mac.
3. Kết quả: binary `aarch64-unknown-linux-gnu`.

Không dùng `cross` hay `cargo-zigbuild`: các thư viện đồ hoạ (`libinput`, `libgbm`, EGL) cần sysroot đầy đủ, container bookworm arm64 có sẵn nên đơn giản hơn.

## 5. Đưa lên Pi

1. Chép binary, model wake word và `deploy/ai-hub.service` sang Pi bằng `scp` (hoặc `rsync`).
2. Lần đầu: cài thư viện runtime (`libinput10`, `libgbm1`, `libegl1`, `libgles2`, `libasound2`, `libfontconfig1`), tạo user `hub` trong nhóm `video input render audio`, bật service.
3. Các lần sau: chép binary mới rồi `systemctl restart ai-hub`.
4. Xem log: `journalctl -u ai-hub -f`.

Gom các bước này vào một script deploy (dự kiến `scripts/deploy-pi.sh <host>`) khi dựng repo.

## 6. Chỉ kiểm tra được trên Pi thật

| Hạng mục | Vì sao |
|---|---|
| Backend `linuxkms`, renderer FemtoVG trên GPU V3D | Mac không có DRM/KMS |
| **fps thực tế của hiệu ứng** (Orb, chữ rung, nền thời tiết, chuyển màn) | GPU Pi yếu hơn nhiều; đo sớm trước khi làm nhiều hiệu ứng |
| Màn cảm ứng thật: hướng xoay, độ nhạy, cảm giác vuốt | Chuột không giống ngón tay |
| Kiosk: systemd, thời gian boot, splash, ẩn console | Chỉ có trên Pi OS |
| Lần đầu khởi động: điểm truy cập Wi-Fi + 2 mã QR | Cần Wi-Fi của Pi |
| Nhiệt độ và CPU khi chạy lâu (wake word + UI) | Pi có thể hạ xung khi nóng |

## 7. Thứ tự gợi ý

1. Dựng workspace + cửa sổ Slint trống chạy trên Mac.
2. Build bản Pi trong Docker và chạy được màn trống trên Pi (kiểm chứng sớm chuỗi build + `linuxkms`).
3. `voice-client` nói chuyện được với vela-server cục bộ.
4. Orb + Voice dock + wake word trên Mac; đo fps hiệu ứng trên Pi.
5. Lần lượt các màn: trung tâm, Menu, Thời tiết, Lịch, Hẹn giờ, Đếm ngược, Reminder, Ảnh, Setting.
