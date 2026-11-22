# AI Hub — Thiết kế codebase

> Trạng thái: **thiết kế**, repo chưa có code. Cập nhật: 2026-11-22.
>
> Ý tưởng sản phẩm: [concept.md](concept.md). Phát triển trên Mac: [development.md](development.md). Quyết định đã chốt: [adr/](adr/). Từ vựng: [GLOSSARY.md](../GLOSSARY.md).

## 1. Nguyên tắc

- **Ba crate, phụ thuộc một chiều** `voice-client ← hub-core ← apps/hub` ([ADR 0001](adr/0001-three-crate-workspace.md)).
  - `hub-core` chứa **toàn bộ hành vi** của hub. Không Slint, không thiết bị âm thanh, không lời gọi OS. HTTP client, SQLite, Setup web được phép vì build thuần trên Mac.
  - `apps/hub` chỉ còn adapter cần thư viện hệ thống hoặc phần cứng (Slint, `cpal`, libopus, ONNX Runtime, backlight, NetworkManager) và phần nối dây.
- **Một actor giữ trạng thái**, UI chỉ đọc `UiState` ([ADR 0002](adr/0002-single-process-actor-ui-reads-snapshot.md)).
- **Seam chỉ đặt chỗ có ít nhất hai adapter.** Nguồn dữ liệu bên ngoài và OS có adapter thật + fake/desktop; `Store` và `ConfigStore` chỉ có một adapter nên không có trait.
- **Kênh là seam giữa các crate.** `hub-core` nói với voice, audio và UI qua `mpsc`/`watch` với enum lệnh/sự kiện; test chỉ cần tạo channel.
- **Thời gian luôn đi vào từ ngoài** (port `Clock`, hoặc tham số `now` cho hàm thuần). Không gọi giờ hệ thống bên trong logic tính năng.

## 2. Cấu trúc thư mục

```
ai-hub/
├── Cargo.toml                    # workspace + [workspace.dependencies]
├── Cargo.lock
├── rust-toolchain.toml           # 1.98
├── .gitignore                    # target/, models/, .data/
├── AGENTS.md
├── GLOSSARY.md
│
├── crates/
│   ├── voice-client/             # Voice Protocol tới vela-server
│   │   ├── src/
│   │   │   ├── lib.rs            # spawn(), VoiceConfig, VoiceCommand, VoiceEvent, ToolCall
│   │   │   ├── protocol.rs       # message JSON (serde), private
│   │   │   ├── connection.rs     # WS, ClientHello/ServerHello, reconnect backoff
│   │   │   ├── enrollment.rs     # OTA/Enrollment: mã kích hoạt → URL WS + token
│   │   │   └── mcp.rs            # JSON-RPC: initialize, tools/list, tools/call
│   │   └── tests/
│   │       ├── support/fake_server.rs   # server WS giả, kịch bản message
│   │       └── *.rs
│   │
│   └── hub-core/                 # toàn bộ hành vi của hub
│       ├── src/
│       │   ├── lib.rs            # run(), Io, Ports, re-export type công khai
│       │   ├── runtime.rs        # actor: select! qua mọi input, giữ HubState
│       │   ├── conversation.rs   # Idle / Listening / Thinking / Speaking / Offline
│       │   ├── view.rs           # điều hướng, focus, tự quay về, Ambient, "tạm xem"
│       │   ├── alerts.rs         # trạng thái báo toàn màn: hàng đợi, tự tắt, đợi hết lượt nói
│       │   ├── ui_state.rs       # UiState, UserInput
│       │   ├── audio.rs          # AudioCommand, AudioEvent (adapter ở apps/hub)
│       │   ├── tools/
│       │   │   ├── mod.rs        # definitions(), dispatch
│       │   │   └── args.rs       # struct tham số từng hub_* (serde + schemars)
│       │   ├── server_tools.rs   # bảng tên tool phía server → màn
│       │   ├── features/
│       │   │   ├── timers.rs
│       │   │   ├── countdowns.rs
│       │   │   ├── reminders.rs
│       │   │   ├── weather.rs    # vị trí nhà / tạm xem, WMO → icon & nền
│       │   │   ├── calendar.rs   # cache 60 ngày, syncToken, sự kiện mới tạo
│       │   │   ├── home.rs       # entity demo, trạng thái cũ
│       │   │   ├── photos.rs     # 100 ảnh / 200 MB, ghim, xoá cũ nhất
│       │   │   └── selector.rs   # chọn mục theo nhãn → Found / Ambiguous / NotFound
│       │   ├── lunar.rs          # âm–dương lịch Việt Nam (UTC+7)
│       │   ├── ports.rs          # Clock, WeatherSource, CalendarSource, HomeSource, ImageFetcher, System
│       │   ├── sources/          # adapter thật của port (chạy được cả Mac và Pi)
│       │   │   ├── open_meteo.rs       # forecast, air quality, geocoding
│       │   │   ├── google_calendar.rs  # events.list/insert, OAuth đổi mã + refresh
│       │   │   ├── home_assistant.rs   # WebSocket API
│       │   │   └── image_fetch.rs      # giới hạn, chặn LAN/localhost, thu nhỏ
│       │   ├── store/
│       │   │   ├── mod.rs        # Store::open(dir) / Store::in_memory()
│       │   │   └── migrations/   # 0001_init.sql, …
│       │   ├── config.rs         # ConfigStore: TOML 0600, watch
│       │   └── setup_web/
│       │       ├── mod.rs        # serve(addr, …)
│       │       └── pages/        # HTML tĩnh
│       └── tests/
│           ├── support/          # harness: dựng run() với channel, fake port, test clock
│           ├── fixtures/         # JSON thật của Open-Meteo, Google, HA
│           └── *.rs              # voice_flow, timers, reminders, calendar, photos, view…
│
├── apps/hub/                     # binary chạy trên Mac và Pi
│   ├── Cargo.toml                # features: desktop (mặc định) | pi
│   ├── build.rs                  # slint-build
│   ├── src/
│   │   ├── main.rs               # nối dây (xem 3.3)
│   │   ├── ui/
│   │   │   ├── mod.rs            # run_ui(): event loop Slint trên main thread
│   │   │   └── bridge.rs         # UiState → struct Slint; callback Slint → UserInput
│   │   ├── audio/
│   │   │   ├── mod.rs            # spawn() → (Sender<AudioCommand>, Receiver<AudioEvent>, watch<Levels>)
│   │   │   ├── devices.rs        # cpal: chọn XVF3800 cả vào lẫn ra
│   │   │   ├── capture.rs        # mic → wake word + preroll + Opus encode
│   │   │   ├── wake.rs           # openWakeWord qua ort
│   │   │   ├── preroll.rs        # ring buffer sau wake word
│   │   │   └── playback.rs       # Opus decode + mixer (TTS, chuông, âm giao diện)
│   │   └── system/
│   │       ├── mod.rs            # chọn adapter theo feature
│   │       ├── pi.rs             # backlight sysfs, nmcli, systemctl
│   │       └── desktop.rs        # no-op + log
│   ├── ui/                       # file .slint
│   │   ├── app.slint             # cửa sổ gốc: tầng màn, Voice dock, lớp báo
│   │   ├── theme.slint           # màu, font, cỡ theo scale-factor
│   │   ├── components/           # orb, response-text, tool-trail, card, slider, page-dots
│   │   └── screens/              # center, menu, weather, calendar, home, timers,
│   │                             # countdowns, reminders, photos, settings, ambient,
│   │                             # alert, first-boot
│   └── assets/
│       ├── fonts/                # Be Vietnam Pro / Inter, 2 độ đậm
│       ├── icons/                # SVG outline
│       └── sounds/               # ding, chuông hẹn giờ, chuông Reminder
│
├── models/                       # ONNX wake word — không commit
├── docker/pi-build.Dockerfile
├── deploy/ai-hub.service
├── scripts/
│   ├── fetch-models.sh
│   └── deploy-pi.sh
└── docs/
    ├── concept.md
    ├── development.md
    ├── codebase.md
    ├── adr/
    └── agents/
```

## 3. Module và interface

### 3.1 `voice-client`

Một module sâu: toàn bộ Voice Protocol nằm sau hai channel.

```rust
pub fn spawn(config: watch::Receiver<VoiceConfig>)
    -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>);

pub struct VoiceConfig {
    pub server: ServerConfig,     // URL OTA; URL WS + token nếu đã Enrollment
    pub device: DeviceIdentity,
    pub features: Features,       // mcp, tool_status, direct_tts, aec = true
    pub tools: Vec<ToolDef>,      // trả lời tools/list
}

pub enum VoiceCommand {
    ListenStart,                  // listen:start, mode realtime
    ListenStop,
    Audio(OpusFrame),             // bỏ khi chưa Ready
    DirectSpeech { request_id: String, text: String },
    PlaybackAck { .. },
}

pub enum VoiceEvent {
    Connection(ConnectionState),  // Connecting | Ready | Offline { retry_in }
    Activation { code: String },  // chưa Enrollment: hiện mã trên màn
    Enrolled(Credentials),        // hub-core ghi vào config
    Stt { text: String },
    TtsStart { origin: TtsOrigin },          // Agent | ClientDirect { request_id }
    TtsText { text: String },
    TtsAudio(OpusFrame),
    TtsStop,
    DirectSpeechRejected { request_id: String, reason: String },
    ServerTool { name: String, state: ToolState },  // started | completed | failed
    ToolCall(ToolCall),
}

pub struct ToolCall { pub name: String, pub arguments: serde_json::Value, /* reply */ }
impl ToolCall { pub fn reply(self, result: Result<serde_json::Value, ToolError>); }
```

**Giấu bên trong:** WebSocket, handshake, admission, Enrollment, reconnect có backoff, đánh số JSON-RPC, trả lời `tools/list`, frame text vs binary, kết nối lại khi `VoiceConfig` đổi (vd. đổi địa chỉ server qua Setup web). `ToolCall` bị drop mà chưa `reply` → voice-client tự trả lỗi nội bộ, server không treo.

**Test:** server WS giả trong `tests/support`, kịch bản message theo `docs/api/studio-assistant.md` của vela-server; backoff với tokio time paused.

### 3.2 `hub-core`

#### Interface bên ngoài

```rust
pub async fn run(config: ConfigStore, store: Store, ports: Ports, io: Io) -> anyhow::Result<()>;

pub struct Io {
    pub voice_tx: mpsc::Sender<VoiceCommand>,
    pub voice_rx: mpsc::Receiver<VoiceEvent>,
    pub audio_tx: mpsc::Sender<AudioCommand>,
    pub audio_rx: mpsc::Receiver<AudioEvent>,
    pub input_rx: mpsc::Receiver<UserInput>,
    pub ui_tx: watch::Sender<UiState>,
}

pub struct Ports {
    pub clock: Arc<dyn Clock>,
    pub weather: Arc<dyn WeatherSource>,
    pub calendar: Arc<dyn CalendarSource>,
    pub home: Arc<dyn HomeSource>,
    pub images: Arc<dyn ImageFetcher>,
    pub system: Arc<dyn System>,
}

pub fn tool_definitions() -> Vec<ToolDef>;   // apps/hub đưa vào VoiceConfig
```

Gần như mọi hành vi của sản phẩm — luồng voice, 21 tool `hub_*`, điều hướng, báo hết giờ, Reminder, cache offline, Setup web — test được qua đúng interface này: gửi `VoiceEvent`/`AudioEvent`/`UserInput`, tiến clock, rồi kiểm `VoiceCommand`/`AudioCommand` đã gửi và `UiState`.

`run` tự spawn Setup web, các vòng làm mới dữ liệu (thời tiết 30 phút, lịch 5 phút, kết nối HA) và actor.

#### Kênh audio

```rust
pub enum AudioCommand {
    Mic { on: bool },              // on: true → audio xả preroll trước rồi stream tiếp
    PlayTts(OpusFrame), StopTts,
    Play(Sound),                   // Ding, TimerChime, ReminderChime, ToolDone
    StopSound(Sound),
    SetVolume(u8), SetWakeWord { enabled: bool, sensitivity: Sensitivity },
}

pub enum AudioEvent {
    WakeWord,
    MicFrame(OpusFrame),
    TtsPlaybackFinished,           // → PlaybackAck
}
```

Biên độ mic và năng lượng TTS không đi qua `hub-core`: audio ghi thẳng vào `watch<Levels>` mà UI đọc.

#### Module bên trong

Actor (`runtime.rs`) nhận mọi input, gọi các module dưới đây, rồi phát `UiState` mới (`send_if_modified`). Mỗi module có seam nội bộ riêng để unit test, nhưng không phải interface của crate.

| Module | Giữ gì | Interface chính |
|---|---|---|
| `conversation` | Trạng thái voice, chờ im lặng 10 s sau TTS, mic tắt | Hàm thuần `on(input, now) -> Vec<Action>`; `turn_active()` |
| `view` | Màn hiện tại, focus, tự về màn trung tâm sau ~60 s, Ambient, "tạm xem" | `open(screen, focus)`, `on_user(..)`, `tick(now)` |
| `alerts` | Hàng đợi trạng thái báo (hết giờ, Reminder): lần lượt, tự tắt 2 phút / 60 s, đợi hết lượt nói | `push(alert)`, `dismiss()`, `tick(now, turn_active)` |
| `tools` | Tên + JSON Schema + parse tham số của từng `hub_*` | `definitions()`, `parse(name, args) -> ToolRequest` |
| `features::timers` | ≤ 4 hẹn giờ, giờ kết thúc tuyệt đối, tạm dừng | `start`, `adjust`, `cancel`, `due(now)` |
| `features::countdowns` | ≤ 8 mốc, `none` / `yearly` / `yearly_lunar` | `create`, `remove`, `next_occurrence(now)` |
| `features::reminders` | `none` / `daily` / `weekly`, đã nhắc, đã lỡ (< 1 giờ thì nhắc một lần) | `create`, `cancel`, `snooze`, `due(now)` |
| `features::weather` | Vị trí nhà, thành phố tạm xem (giữ ~30 phút), WMO → icon/nền | `snapshot(focus)`, `apply(forecast)` |
| `features::calendar` | Cache ~60 ngày, `syncToken`, sự kiện mới tạo | `events(range)`, `apply(delta)` |
| `features::photos` | Chỉ mục ảnh, ghim, chính sách xoá | `add`, `pin`, `evict()` |
| `selector` | Chọn mục theo nhãn (dùng chung cho hẹn giờ, mốc, Reminder) | `select(items, query) -> Found / Ambiguous(Vec) / NotFound` |
| `lunar` | Thuật toán âm lịch Việt Nam (Hồ Ngọc Đức), múi UTC+7 | `to_solar(lunar_date)`, `to_lunar(date)` |
| `server_tools` | Tên tool phía server → màn | `screen_for(name) -> Option<Screen>` |
| `store` | SQLite + thư mục ảnh ([ADR 0012](adr/0012-sqlite-for-local-data.md)) | method theo tính năng, mỗi thao tác một transaction |
| `config` | File TOML 0600, thông báo khi đổi | `get()`, `update(f)`, `watch()` |

Tính năng là **logic thuần nhận `now`, trả kết quả**; actor lo ghi `Store` và phát hiệu ứng. Vì vậy test một luật (vd. Reminder `weekly` qua Chủ nhật) không cần actor.

#### Seam bên ngoài (port)

| Port | Adapter thật | Adapter khác |
|---|---|---|
| `Clock` | `SystemClock` (hub-core) | Clock test theo tokio time paused |
| `WeatherSource` | `sources::open_meteo` | Fake trong `tests/support` |
| `CalendarSource` | `sources::google_calendar` | Fake |
| `HomeSource` | `sources::home_assistant` | Fake |
| `ImageFetcher` | `sources::image_fetch` | Fake |
| `System` (độ sáng, Wi-Fi, AP setup, khởi động lại/tắt máy) | `apps/hub/system/pi.rs` | `system/desktop.rs`, fake |

Không phải seam (một adapter): `Store` (test dùng `Store::in_memory()`), `ConfigStore` (test dùng thư mục tạm), `setup_web`, OAuth Google (nằm trong `sources::google_calendar`, Setup web gọi trực tiếp).

### 3.3 `apps/hub`

`main.rs` chỉ nối dây:

1. Đọc thư mục dữ liệu, mở `ConfigStore` và `Store`.
2. `voice_client::spawn(...)` với `VoiceConfig` dẫn từ `config.watch()` và `hub_core::tool_definitions()`.
3. `audio::spawn(...)` trên thread riêng.
4. Dựng `Ports` (adapter `sources::*` của hub-core, `system` theo feature).
5. Chạy tokio runtime ở thread phụ với `hub_core::run(...)`.
6. Chạy `ui::run_ui(ui_rx, levels_rx, input_tx)` trên main thread (Slint `linuxkms` cần main thread).

**UI bridge:** `bridge.rs` đổi `UiState` sang struct Slint qua `slint::invoke_from_event_loop`, và callback Slint sang `UserInput`. Logic không nằm ở đây; nếu bridge cần một nhánh `if` theo nghiệp vụ thì nhánh đó thuộc `hub-core`.

**Audio:** một luồng PCM 16 kHz mono từ XVF3800 chia hai: khối 80 ms cho openWakeWord (melspectrogram → embedding → `hey_jarvis`) và khối 60 ms cho Opus encode. `preroll` giữ ~1–2 s gần nhất để xả khi `Mic { on: true }`. `playback` trộn TTS với chuông (chuông nhỏ hơn khi đang có TTS), mọi âm ra đều qua cổng ra của mic array ([ADR 0005](adr/0005-local-wake-word-hardware-aec.md)).

## 4. Luồng qua các module

**Wake word → nói:**

```
audio: AudioEvent::WakeWord
  → conversation.on(Wake): Idle → Listening
  → actor gửi VoiceCommand::ListenStart, AudioCommand::Play(Ding), AudioCommand::Mic{on:true}
audio: MicFrame (preroll trước) → actor → VoiceCommand::Audio (chỉ khi conversation đang nghe)
voice: TtsStart / TtsAudio / TtsStop → AudioCommand::PlayTts …; UiState.voice cập nhật Orb, chữ phản hồi
10 s im lặng sau TtsStop → conversation: → Idle → ListenStop + Mic{on:false}
```

**Tool `hub_*`:**

```
voice: VoiceEvent::ToolCall
  → tools::parse → ToolRequest (lỗi tham số → reply lỗi ngay)
  → phần đồng bộ: view.open(screen, focus), cập nhật tính năng, ghi Store, thêm spawn vào UiState
  → phần cần mạng (calendar_create, weather location khác, show_image): spawn task dùng port,
    kết quả quay về actor như một message nội bộ → hoàn tất trạng thái → ToolCall::reply
```

**Reminder đến hạn:**

```
tick 1 Hz → reminders.due(now) → alerts.push(Reminder)
  → đợi conversation.turn_active() == false
  → UiState.alert, AudioCommand::Play(ReminderChime)
  → online và bật đọc bằng giọng: VoiceCommand::DirectSpeech{request_id}
      DirectSpeechRejected(session_busy) → thử lại sau; Offline → chỉ chuông + chữ
  → ~60 s → alert vào "Đã nhắc gần đây"
```

Actor tick 1 Hz cho mọi thứ theo giờ (hẹn giờ, Reminder, tự quay về, chữ số đếm ngược); vòng tròn hẹn giờ mượt nhờ Slint `animate` tuyến tính 1 s giữa hai tick.

## 5. Dữ liệu trên thiết bị

| Nơi chạy | Thư mục dữ liệu |
|---|---|
| Pi | `/var/lib/ai-hub/` (`StateDirectory=ai-hub` trong systemd) |
| Mac (dev) | `./.data/` hoặc `AI_HUB_DATA_DIR` |

```
<data>/
├── config.toml        # 0600: server, token, Google refresh token, HA, Wi-Fi, vị trí, setting
├── hub.sqlite         # hẹn giờ, mốc, Reminder, cache thời tiết/lịch/HA, chỉ mục ảnh
└── photos/            # ảnh đã thu nhỏ + thumbnail
```

Model wake word: `models/` cạnh binary (Pi: `/opt/ai-hub/models/`).

## 6. Test

| Tầng | Cách test | Chạy ở |
|---|---|---|
| Hàm thuần (`lunar`, `selector`, luật lặp, `conversation`, `view`) | Unit test, truyền `now` | `cargo test -p hub-core` |
| Hành vi hub | `tests/` của hub-core: harness dựng `run()` với channel, fake port, `Store::in_memory()`, tokio time paused | Mac |
| Adapter `sources::*` | Parse fixture JSON thật; không gọi mạng trong CI | Mac |
| `voice-client` | Server WS giả | Mac |
| `apps/hub` | Unit test `preroll`; còn lại chạy tay | Mac + Pi |
| fps, cảm ứng, kiosk, AP setup | Thủ công trên Pi ([development.md §6](development.md)) | Pi |

Ví dụ test qua interface của hub-core: gửi `ToolCall("hub_timer_start", {minutes: 7, label: "luộc trứng"})` → kiểm reply OK, `UiState` ở màn Hẹn giờ có một thẻ; tiến clock 7 phút → `UiState.alert` là hết giờ và có `AudioCommand::Play(TimerChime)`.

## 7. Câu hỏi mở phát hiện khi thiết kế

- **Giờ trước khi NTP đồng bộ.** Pi 4 không có RTC; khởi động khi mất mạng thì giờ hệ thống sai (fake-hwclock chỉ giữ giờ lúc tắt). Hẹn giờ/Reminder theo giờ tuyệt đối có thể báo sai hoặc gắn "Đã lỡ" sai. Đề xuất: `System` báo trạng thái đồng bộ giờ, `alerts` không xử lý mục quá hạn cho tới khi đã đồng bộ.
- **Đổi độ phân giải lúc chạy.** Chưa rõ Slint `linuxkms` có đổi mode KMS mà không khởi động lại app không. Nếu phải khởi động lại, luồng "Giữ độ phân giải này? 10" cần lưu mode cũ vào config để quay về sau lần khởi động kế.
- **Sample rate.** Kiểm sample rate Opus mà Voice Protocol yêu cầu; openWakeWord cần 16 kHz. Khác nhau thì `capture` phải resample một nhánh.
