# AI Hub — Phát triển trên máy Mac

> Trạng thái: **kế hoạch**, repo chưa có code. Lệnh và đường dẫn dưới đây là cách dự kiến tổ chức; cập nhật lại khi dựng workspace. Cập nhật: 2026-11-24.

Xem thiết kế tổng thể ở [concept.md](concept.md).

## 1. Máy phát triển

| Thành phần | Giá trị |
|---|---|
| Máy | Mac Apple Silicon (arm64), 16 GB RAM |
| Rust | 1.98 (rustup), target `aarch64-apple-darwin` |
| Thiết bị đích | Máy tính bảng Fujitsu Q5010 (x86_64), Debian 13 trixie không desktop; màn cảm ứng, mic và loa tích hợp |

Ý chính: **viết và chạy hub ngay trên Mac** (hoặc trên Linux có desktop), rồi build bản kiosk ngay trên máy tính bảng bằng `scripts/install-linux.sh`.

## 2. Cấu trúc repo dự kiến

```
ai-hub/
├── Cargo.toml              # workspace
├── crates/voice-client/    # kết nối Voice Protocol, không UI, không thiết bị âm thanh
├── crates/hub-core/        # hành vi của hub, test bằng `cargo test -p hub-core`
├── apps/hub/               # app Slint chạy trên Mac và máy tính bảng Linux
│   └── ui/                 # file .slint
├── deploy/ai-hub.service   # systemd cho máy tính bảng
└── docs/
```

## 3. Chạy trên Mac

Những gì app thật trên Mac chưa làm được: [limitations.md](limitations.md). Kế hoạch hoàn thiện bản Mac: [.scratch/hub-mac/spec.md](../.scratch/hub-mac/spec.md).

### Backend Slint theo nơi chạy

| Nơi chạy | Feature Slint | Ghi chú |
|---|---|---|
| Mac / Linux desktop (phát triển) | `backend-winit` | Cửa sổ desktop; chuột thay ngón tay |
| Máy tính bảng (kiosk) | `backend-linuxkms` + `renderer-femtovg` | Vẽ thẳng ra DRM/KMS, không X11/Wayland |

Chọn bằng feature của crate `apps/hub`: `desktop` mặc định, `kiosk` khi build cho máy tính bảng. Code ngoài phần chọn backend giống nhau trên hai nơi.

### Mô phỏng màn hình máy tính bảng

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

- TTS Opus cần thư viện hệ thống: cài bằng `brew install opus` trước khi build.
- Lần đầu dùng mic, macOS hỏi quyền Microphone cho terminal/IDE đang chạy app.
- Wake word `hey_jarvis` dùng openWakeWord qua `ort`. Ba model ONNX được đọc từ `AI_HUB_MODELS_DIR`, mặc định là `models/`; model không commit.
- Lần đầu mở app, nếu thiếu model, app tự tải openWakeWord v0.5.1 (~3,7 MB) từ GitHub vào thư mục này ở nền; wake word bật ngay khi tải xong. Không có mạng thì wake word tắt, lần mở sau tải lại.
- Trong worktree ticket/integration hiện tại, chạy app với bộ model đã có ở checkout chính:

  ```sh
  AI_HUB_MODELS_DIR="$(git rev-parse --path-format=absolute --git-common-dir)/../models" scripts/run.sh run
  ```

### Dịch vụ bên ngoài

Open-Meteo, Google Calendar API, Home Assistant và Setup web chạy trên Mac giống hệt máy tính bảng.

- Google OAuth: khi dev trên Mac, trình duyệt chạy cùng máy nên redirect `http://127.0.0.1:<port>` về thẳng app — không cần bước chép URL như trên máy tính bảng.
- Home Assistant: trỏ tới HA trong mạng nhà, hoặc chạy HA bằng Docker để demo.

## 4. Build và cài trên máy tính bảng

Build ngay trên máy tính bảng (Debian 13 trixie, x86_64), không cross-compile từ Mac (ADR 0014).

1. Clone repo lên máy tính bảng.
2. Lần đầu: `scripts/install-linux.sh --init` — cài thư viện build (`libudev-dev`, `libinput-dev`, `libdrm-dev`, `libgbm-dev`, `libegl-dev`, `libgles-dev`, `libxkbcommon-dev`, `libfontconfig-dev`, `libseat-dev`, `libasound2-dev`, `libopus-dev`), Rust qua rustup, user `hub`, rồi bật service.
3. Các lần sau: `git pull` rồi `scripts/install-linux.sh` — build `cargo build --release -p hub --no-default-features --features kiosk` vào `target/kiosk`, chép binary vào `/opt/ai-hub`, khởi động lại service.
4. Xem log: `journalctl -u ai-hub -f`.

Service đặt `AI_HUB_MODELS_DIR=/var/lib/ai-hub/models`; lần đầu chạy app tự tải model vào đó (cần mạng).

## 5. Âm thanh trên máy tính bảng

Mic và loa tích hợp, chọn thiết bị ALSA mặc định. Máy không có AEC phần cứng như XVF3800 (xem ADR 0005 và [limitations.md](limitations.md)).

## 6. Chỉ kiểm tra được trên máy tính bảng thật

| Hạng mục | Vì sao |
|---|---|
| Backend `linuxkms`, renderer FemtoVG trên GPU Intel | Mac không có DRM/KMS |
| **fps thực tế của hiệu ứng** (Orb, chữ rung, nền thời tiết, chuyển màn) | Đo sớm trước khi làm nhiều hiệu ứng |
| Màn cảm ứng thật: hướng xoay, độ nhạy, cảm giác vuốt | Chuột không giống ngón tay |
| Mic/loa tích hợp: wake word và barge-in khi loa đang phát | Không có AEC phần cứng |
| Kiosk: systemd, thời gian boot, ẩn console | Chỉ có trên máy thật |
| Lần đầu khởi động: điểm truy cập Wi-Fi + 2 mã QR | Cần Wi-Fi của máy |
| Nhiệt độ, CPU và pin khi chạy lâu (wake word + UI) | Máy có thể hạ xung khi nóng |

## 7. Thứ tự gợi ý

1. Dựng workspace + cửa sổ Slint trống chạy trên Mac.
2. Build bản kiosk trên máy tính bảng và chạy được màn trống (kiểm chứng sớm chuỗi build + `linuxkms`).
3. `voice-client` nói chuyện được với vela-server cục bộ.
4. Orb + Voice dock + wake word trên Mac; đo fps hiệu ứng trên máy tính bảng.
5. Lần lượt các màn: trung tâm, Menu, Thời tiết, Lịch, Hẹn giờ, Đếm ngược, Reminder, Ảnh, Setting.
