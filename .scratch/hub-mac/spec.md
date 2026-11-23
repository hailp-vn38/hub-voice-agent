# Spec: hub dùng được trên macOS

Status: ready-for-agent

## Mục tiêu

Chạy `ai-hub` (feature `desktop`, không `demo`) trên Mac và dùng được hằng ngày, bằng chạm lẫn giọng nói:

- Mọi màn Aurora hiển thị dữ liệu thật từ hub-core, thao tác chạm đi về hub-core.
- Nói "hey jarvis" hoặc chạm Orb để bắt đầu lượt nói; server nghe được, TTS phát ra loa; tool `hub_*` mở đúng màn.
- Hẹn giờ và Reminder kêu chuông cho tới khi tắt.

Pi làm sau, khi bản Mac đã ổn ([§ Ngoài phạm vi](#ngoài-phạm-vi)).

Thiết kế chi tiết: [design.md](design.md). Hạn chế còn lại sau spec này: [docs/limitations.md](../../docs/limitations.md).

## Hiện trạng

- `hub-core` đã có lớp `present(&UiState) -> HubView` (ticket hub-core 14–21) và các `UserInput` cần cho chạm. Không cần đổi hub-core cho spec này.
- UI Aurora, hợp đồng `ui/contract/*.slint` và demo đã ở `main`.
- Bridge thật chỉ có `StatusBar.time` và phần Kết nối voice; `Nav` đổi màn ngay trong Slint.
- Audio chưa có: `main.rs` bỏ mọi `AudioCommand`, không gửi `AudioEvent`.
- Âm thanh: `apps/hub/assets/audio/ping.mp3`, `reminder.mp3` (chưa commit, chưa ghi nguồn).

## Interface đích (`apps/hub`)

```rust
// src/ui/bridge/mod.rs
pub fn spawn(app: Weak<App>, ui_rx: watch::Receiver<UiState>, levels_rx: watch::Receiver<Levels>);
pub fn connect(app: &App, input_tx: mpsc::Sender<UserInput>);

// src/audio/mod.rs
pub fn spawn(config: AudioConfig) -> anyhow::Result<AudioHandle>;
pub struct AudioHandle {
    pub commands: mpsc::Sender<AudioCommand>,
    pub events: mpsc::Receiver<AudioEvent>,
    pub levels: watch::Receiver<Levels>,
}
```

`main.rs` chỉ nối dây: `audio::spawn` → `Io.audio_tx/audio_rx` và `bridge::spawn(.., levels)`.

## Quyết định đã chốt

1. **Mac trước, Pi sau.** Không làm gì chỉ để chạy trên Pi; chỉ không làm gì cản Pi (chọn thiết bị qua `devices.rs`, không giả định CoreAudio ngoài đó).
2. **Tiếng vọng trên Mac:** đeo tai nghe, theo [ADR 0005](../../docs/adr/0005-local-wake-word-hardware-aec.md) và [development.md](../../docs/development.md) "Âm thanh trên Mac". Không tắt mic khi loa đang phát.
3. **Bridge không có nhánh nghiệp vụ.** Trạng thái chỉ thuộc trình bày (lịch xem tháng, chi tiết đang mở, chọn mốc đếm ngược, tự chuyển ảnh) do bridge giữ; mọi thứ khác lấy từ `HubView`.
4. **`Nav.screen` chỉ lấy từ hub-core.** Thêm `Nav.activity()` vào hợp đồng để mọi chạm đặt lại hạn 60 s.
5. **Audio là hai module thuần** `Capture` và `Mixer`, test bằng mảng mẫu; seam duy nhất là `WakeDetector`.
6. **Chuông:** `Ding` = `ping.mp3` (một lần); `TimerChime`, `ReminderChime` = `reminder.mp3` lặp tới `StopSound`; `ToolDone` bỏ qua. Giải mã bằng `symphonia`.
7. **Wake word:** luôn bật, độ nhạy vừa; thiếu model thì chạy không có wake word và ghi log.

## Lát dọc (ticket)

| # | Ticket | Phụ thuộc |
|---|---|---|
| 01 | Bridge khung: Nav + activity, StatusBar, Voice, Response, ToolTrail, Kết nối voice | — |
| 02 | Bridge Hẹn giờ + lớp báo | 01 |
| 03 | Bridge Đếm ngược + Reminder | 01 |
| 04 | Bridge Thời tiết + Lịch | 01 |
| 05 | Bridge Nhà + Ảnh | 01 |
| 06 | Bridge Trung tâm + Menu + Ambient | 02, 03, 04, 05 |
| 07 | Audio ra: Mixer, chuông, TTS | — |
| 08 | Audio vào: Capture, gửi tiếng mic | 01, 07 |
| 09 | Wake word openWakeWord | 08 |
| 10 | Mức âm cho Orb | 01, 08 |

02–05 và 07 làm song song được sau 01. Hub "dùng cơ bản trên Mac" khi 01–09 xong; 10 là hiệu ứng.

## Ngoài phạm vi

- Mọi việc riêng của Pi: XVF3800, `linuxkms`/FemtoVG, fps, image Docker (`libopus`, `ort` aarch64), kiosk, lần đầu khởi động (G10).
- Phần Setting còn thiếu (G3–G8), `hub_open_screen`, `hub_show_card` (G1, G2): xem [../hub-ui/hub-core-gaps.md](../hub-ui/hub-core-gaps.md).
- Thay đổi hub-core (vd. dùng `TtsPlaybackFinished`, `Word.emphasis`).

## Quy ước

- Mỗi ticket bridge điền bảng "Map với hub-core" của ticket hub-ui tương ứng và ghi `Map: done` kèm commit.
- Ticket nào gỡ một hạn chế thì sửa [docs/limitations.md](../../docs/limitations.md) trong cùng commit.
- Tách `src/ui/bridge.rs` thành thư mục trong một commit riêng, không đổi hành vi (AGENTS.md).
- Test trong file riêng (`foo_tests.rs` hoặc `foo/tests.rs`), không viết `mod tests { .. }` trong file nguồn.
- Kiểm tay trên Mac: `scripts/run.sh run`, đeo tai nghe, vela-server cục bộ ([development.md §3](../../docs/development.md)).
