# Thiết kế: bước map UI ↔ hub-core và adapter audio thật

Cập nhật: 2026-11-23 (sửa sau review). Liên quan: [spec.md](spec.md), [../hub-ui/spec.md](../hub-ui/spec.md), [../hub-ui/hub-core-gaps.md](../hub-ui/hub-core-gaps.md), [hạn chế hiện tại](../../docs/limitations.md), [../hub-core/spec.md](../hub-core/spec.md), [codebase.md §2, §3.3](../../docs/codebase.md), [ADR 0005](../../docs/adr/0005-local-wake-word-hardware-aec.md).

## 0. Hiện trạng (đã kiểm trên `main` @ 1d3bad9)

- Không còn nhánh `feat/hub-ui`; hợp đồng, màn và demo đã ở `main`. Không cần bước gộp nhánh.
- Lớp `present` đã có: `present(&UiState) -> HubView`. Ticket hub-core 14, 16–21 `completed`; **#15 `ready-for-human`** chỉ vì `Word.emphasis` chờ metadata nhấn từ protocol. `voice`, `response`, `tool_trail` đã dùng được, `emphasis` luôn `false`.
- Mỗi view chỉ nhận focus của tính năng mình (`present/timers.rs` chỉ lấy `Focus::Timer`…). Serial `events.focus` dùng chung, nên màn khác nhận serial mới với `focus` rỗng.
- Bridge hiện có: `StatusBar.time`, `SettingsData.connection` + `ota-url`. `Nav` vẫn đổi màn ngay trong Slint (`connect_inputs`), trái quy tắc 6 của spec.
- **Không có gì gửi `UserInput::Activity`**; hợp đồng `Nav` chỉ có `navigate`. Hệ quả: đang thao tác trên một màn quá 60 s vẫn bị đưa về trung tâm.
- Audio chưa có: `main.rs` rút bỏ mọi `AudioCommand`, không gửi `AudioEvent`.
- Hub-core gửi tới audio: `Mic { on }`, `PlayTts`, `StopTts`, `Play(Ding)` (wake), `Play(TimerChime | ReminderChime)` khi báo, `StopSound(..)` khi tắt báo (chạm, hoặc tự tắt sau 120 s / 60 s). **Không gửi** `SetVolume`, `SetWakeWord`, `Play(ToolDone)`; **không xử lý** `AudioEvent::TtsPlaybackFinished`.
- Âm thanh có sẵn trong `apps/hub/assets/audio/` (chưa commit): `ping.mp3` (48 kHz stereo, ~1 s), `reminder.mp3` (44,1 kHz mono, ~3 s).

## 1. Bridge

### 1.1 Seam và interface

Seam là hợp đồng Slint (`apps/hub/ui/contract/*.slint`), có hai adapter đang dùng: `ui/demo/` và `ui/bridge/`.

Interface của bridge với phần còn lại của `apps/hub`:

```rust
pub fn spawn(app: Weak<App>, ui_rx: watch::Receiver<UiState>, levels_rx: watch::Receiver<Levels>);
pub fn connect(app: &App, input_tx: mpsc::Sender<UserInput>);
```

Bên trong chỉ copy và đổi kiểu sang Slint. Cần nhánh nghiệp vụ thì đưa vào hub-core `present`.

```
ui/bridge/
├── mod.rs        spawn, connect; giữ HubView trước, gọi apply của mục đã đổi
├── inputs.rs     Inputs::send(UserInput) (try_send, log một chỗ); bảng Screen Slint ↔ hub
├── images.rs     giải mã ảnh, cache theo key, chỉ trên luồng UI
├── voice.rs      Nav, StatusBar, Voice, VoiceLevels, Response, ToolTrail, SettingsData.connection
├── timers.rs     TimersData, AlertData
├── countdowns.rs CountdownsData
├── reminders.rs  RemindersData
├── weather.rs    WeatherData
├── calendar.rs   CalendarData
├── home.rs       HomeData
├── photos.rs     PhotosData
└── center.rs     Center, Menu, AmbientData
```

### 1.2 Quyết định

1. **Chỉ áp phần đã đổi.** Mỗi lần `UiState` đổi, gọi `present` một lần trên luồng tokio, so từng mục với `HubView` trước (`PartialEq`), rồi chỉ gửi các mục đã đổi qua `invoke_from_event_loop`. `now` đổi mỗi giây; không so sánh thì mọi `ListModel` bị thay mỗi giây, mất animation và vị trí cuộn.
2. **Nav lấy từ core.** `Nav.screen` chỉ được set từ `HubView.nav`. `Nav.navigate(s)` gửi `UserInput::Open(s)`, không đổi màn tại UI (bỏ code hiện tại trong `connect_inputs`). Trễ một vòng channel, không cần cập nhật trước.
3. **Thêm `Nav.activity()` vào hợp đồng.** Shell gọi khi có chạm hoặc vuốt bất kỳ; bridge gửi `UserInput::Activity`. Demo bỏ qua callback này. Gọi tối đa ~1 lần/giây để không làm đầy channel (giới hạn ở Shell bằng timer, hoặc ở `Inputs`).
4. **Bảng `Screen` ở một chỗ** (`inputs.rs`), có unit test phủ mọi biến thể của `hub_core::Screen`. `first-boot` không có trong hub-core (G10), chỉ tồn tại ở UI.
5. **Focus rỗng.** Kiểm các handler `changed focus-serial` trong Slint bỏ qua khi `focus` rỗng, vì serial tăng cho mọi màn.
6. **Ảnh.** `slint::Image` không `Send`: cache sống trên luồng UI, khoá theo `PhotoView.key`.
   - Lưới: chỉ giải mã `thumbnail_path`.
   - `full`: chỉ giải mã cho ảnh đang xem và hai ảnh kề; các mục khác trong `viewer` để ảnh rỗng.
   - Xoá khỏi cache các key không còn trong `HubView.photos`.
7. **Giữ phần đã map.** Kết nối voice và `ota-url` (với quy tắc chỉ ghi URL khi core đổi) chuyển sang `voice.rs` ở B1, không đợi tới phần Setting.

### 1.3 Thứ tự

| Bước | Hợp đồng | Ghi chú |
|---|---|---|
| B1 | `Nav` (+ `activity`), `StatusBar`, `Voice`, `VoiceLevels`, `Response`, `ToolTrail`, `SettingsData.connection` | `OrbTapped`, `Activity`; `VoiceLevels` = 0 tới A4 |
| B2 | `TimersData`, `AlertData` | `UserInput::Timer(..)`, `DismissAlert` |
| B3 | `CountdownsData`, `RemindersData` | `RemoveCountdown`; kiểm `confetti_serial` |
| B4 | `WeatherData`, `CalendarData` | `Focus` khi chọn ngày; `month-view` và chi tiết đang mở do bridge giữ, không đổi hợp đồng `CalDetail` |
| B5 | `HomeData`, `PhotosData` | `ToggleHome`, `RemovePhoto`, `PinPhoto`, `Focus` cho viewer; `images.rs` |
| B6 | `Center`, `Menu`, `AmbientData` | `Center.toggle-home` → `ToggleHome`; `AmbientData.wake` → `Activity` |
| — | Phần còn lại của `SettingsData`, `FirstBootData` | Chờ G3–G10 |

Mỗi bước: viết `bridge/<feature>.rs`, điền bảng Map ở ticket UI tương ứng, chạy app thật (không `demo`) theo kịch bản của ticket.

## 2. Adapter audio

### 2.1 Interface

```rust
// apps/hub/src/audio/mod.rs
pub struct AudioHandle {
    pub commands: mpsc::Sender<AudioCommand>,
    pub events: mpsc::Receiver<AudioEvent>,
    pub levels: watch::Receiver<Levels>, // ~30 Hz, không qua hub-core
}
#[derive(Clone, Copy, Default)]
pub struct Levels { pub mic: f32, pub tts: f32 } // 0..1
pub fn spawn(config: AudioConfig) -> anyhow::Result<AudioHandle>;
```

`commands`/`events` vào `Io`, `levels` vào bridge. Hub-core không đổi.

### 2.2 Hình dạng: pipeline thuần, thiết bị là lớp mỏng

Phần đáng test là pipeline, không phải thiết bị. Hai module thuần chứa hầu hết hành vi:

```rust
// capture.rs — PCM 16 kHz mono vào, sự kiện ra
impl Capture {
    pub fn new(wake: Box<dyn WakeDetector>) -> Self;
    pub fn apply(&mut self, command: &AudioCommand);       // Mic { on }
    pub fn push(&mut self, pcm: &[i16]) -> Vec<AudioEvent>; // WakeWord, MicFrame
    pub fn level(&self) -> f32;
}
// mixer.rs — lệnh vào, mẫu ra ở tần số của thiết bị
impl Mixer {
    pub fn new(sounds: Sounds, rate: u32) -> Self;
    pub fn apply(&mut self, command: AudioCommand);          // PlayTts, StopTts, Play, StopSound
    pub fn pull(&mut self, out: &mut [f32]);                 // thiếu dữ liệu → im lặng
    pub fn level(&self) -> f32;
}
```

- `Capture` ẩn: chia khối 80 ms cho wake word, preroll ~1,5 s, xả preroll khi `Mic { on: true }`, encode Opus 16 kHz khối 60 ms, chỉ phát `MicFrame` khi mic bật.
- `Mixer` ẩn: decode Opus 24 kHz, resample lên tần số thiết bị, hàng đợi TTS, `StopTts` xoá hàng đợi, trộn chuông (nhỏ hơn khi đang có TTS), lặp và dừng chuông.
- Test đi qua đúng interface này (`push`/`pull` với mảng mẫu), không cần thiết bị giả.

**Seam duy nhất:** `WakeDetector { fn push(&mut self, pcm: &[i16]) -> bool }`, hai adapter: openWakeWord qua `ort`, và bản giả trong test của `Capture`. Thiết bị, codec, preroll, resample không có seam.

```
apps/hub/src/audio/
├── mod.rs        spawn(): thread audio, mở stream, nối Capture/Mixer với channel
├── devices.rs    chọn thiết bị cpal (XVF3800 trên Pi, mặc định trên Mac), mở stream
├── capture.rs    Capture (+ preroll, encode bên trong; tách file khi > 300 dòng)
├── mixer.rs      Mixer (+ decode, resample bên trong)
├── sounds.rs     Sounds: giải mã mp3 lúc khởi động
└── wake.rs       WakeDetector, OpenWakeWord (ort)
```

### 2.3 Luồng chạy

- `cpal::Stream` không `Send` trên macOS: tạo stream và giữ sống trên **thread audio riêng** do `spawn` dựng; không đưa sang tokio.
- Callback cpal không cấp phát, không khoá:
  - vào: ghi mẫu vào ring SPSC (`rtrb`); thread audio đọc, gọi `Capture::push`, gửi `AudioEvent` bằng `blocking_send`.
  - ra: callback gọi `Mixer::pull` trên một `Mixer` riêng của callback; thread audio chuyển lệnh qua ring SPSC thứ hai. Decode Opus làm ở thread audio trước khi đưa vào ring, không trong callback.
- `levels`: thread audio `send_replace` mỗi ~33 ms.
- Tần số: yêu cầu thiết bị vào 16 kHz mono; không được thì nhận tần số gốc và resample xuống 16 kHz (`rubato`). Ra: theo tần số thiết bị, `Mixer` resample TTS (24 kHz) và chuông.

### 2.4 Âm thanh (`apps/hub/assets/audio/`)

| `Sound` | File | Cách phát |
|---|---|---|
| `Ding` | `ping.mp3` | Một lần |
| `TimerChime` | `reminder.mp3` | Lặp, nghỉ 1 s giữa hai lần, tới `StopSound(TimerChime)` |
| `ReminderChime` | `reminder.mp3` | Lặp, nghỉ 1 s, tới `StopSound(ReminderChime)` |
| `ToolDone` | — | Hub-core không gửi; bỏ qua |

- Nhúng bằng `include_bytes!`, giải mã một lần lúc `spawn` bằng `symphonia` (Rust thuần, không cần thư viện hệ thống cho Pi), đổi về mono và tần số thiết bị.
- Lặp vì hub-core chỉ gửi `Play` một lần khi báo bắt đầu và `StopSound` khi tắt.
- Hẹn giờ và Reminder dùng chung `reminder.mp3` tới khi có file riêng cho hẹn giờ; thêm file mới chỉ sửa bảng trong `sounds.rs`.
- Cần ghi nguồn và giấy phép của hai file vào `apps/hub/assets/README.md` trước khi commit.

### 2.5 Ngoài phạm vi adapter

- `SetVolume`, `SetWakeWord`: hub-core không gửi. Wake word luôn bật, độ nhạy vừa; âm lượng dùng của hệ thống. Làm cùng G3/G5.
- `TtsPlaybackFinished`: hub-core bỏ qua; adapter không cần phát. Nếu muốn khoảng 10 s chờ nói tiếp tính từ lúc loa phát xong (thay vì từ `TtsStop` của server), đó là thay đổi hub-core riêng.
- Tiếng vọng trên Mac: theo ADR 0005, đeo tai nghe; không tắt mic khi phát. Trên Pi, AEC của XVF3800 xử lý.

### 2.6 Phụ thuộc cần kiểm

- `opus` cần libopus: thêm vào image Docker bookworm (ADR 0011) và cài trên Mac.
- `ort` trên aarch64: kiểm có binary sẵn hoặc phải build ONNX Runtime trong image.
- Model `melspectrogram.onnx`, `embedding_model.onnx`, `hey_jarvis.onnx` trong `models/` (docs §5); xác nhận giấy phép trước khi commit.

### 2.7 Thứ tự

| Bước | Việc | Kiểm |
|---|---|---|
| A1 | `Mixer` + `sounds` + thiết bị ra | Test `Mixer`: `StopTts` xoá hàng đợi, chuông lặp tới `StopSound`. Tay: báo hẹn giờ kêu, TTS phát được |
| A2 | `Capture` (không wake) + thiết bị vào | Test `Capture`: không `MicFrame` khi mic tắt, preroll xả trước khi mic bật. Tay: chạm Orb (B1), nói, server trả lời |
| A3 | `OpenWakeWord` | Test `Capture` với detector giả; tay: "hey jarvis" bắt đầu lượt |
| A4 | `levels` → `VoiceLevels` | Tay: Orb nhịp theo mic và TTS |

Thứ tự chung: **B1 → A1 → A2 → A3 → A4**, B2–B6 song song với A. B1 đi trước A2 vì A2 cần chạm Orb để kiểm.

## 3. Ticket

Spec và ticket: [spec.md](spec.md), `issues/01`–`10` (B1–B6 = 01–06, A1–A4 = 07–10). Bridge điền thêm bảng "Map với hub-core" của ticket hub-ui tương ứng.
