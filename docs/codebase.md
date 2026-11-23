# AI Hub — Thiết kế codebase

> Trạng thái: **thiết kế**, đang triển khai. Cập nhật: 2026-11-23 (theo [spec hub-core](../.scratch/hub-core/spec.md)).
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
│   │   │   ├── reconnect.rs      # task client: vòng discover → connect, backoff, jitter
│   │   │   ├── ota.rs            # OTA discovery: URL WS + token
│   │   │   ├── protocol.rs       # wire format: hello, frame uplink, parse downlink (serde), private
│   │   │   ├── connection.rs     # Client, Failure, request WS, ClientHello/ServerHello
│   │   │   ├── connection/
│   │   │   │   ├── timeouts.rs   # Timeouts: OTA, handshake, ping, idle, ngưỡng ổn định
│   │   │   │   └── voice.rs      # vòng Ready: command, frame, lifecycle TTS, reply MCP
│   │   │   ├── mcp.rs            # JSON-RPC: initialize, tools/list, tools/call
│   │   │   └── *_tests.rs, */    # test của từng module (connection_tests, mcp_tests/, reconnect_tests/…)
│   │   │                         # enrollment qua OTA activation
│   │   └── tests/
│   │       ├── support/fake_server.rs   # server WS giả, kịch bản message
│   │       └── *.rs
│   │
│   └── hub-core/                 # toàn bộ hành vi của hub
│       ├── src/
│       │   ├── lib.rs            # run(), Io, Ports, re-export type công khai
│       │   ├── runtime/          # actor: select! qua mọi input, giữ HubState
│       │   │   ├── mod.rs        # vòng select!
│       │   │   ├── voice.rs, audio.rs, input.rs, tick.rs
│       │   │   ├── tools.rs      # định tuyến ToolCall theo tiền tố tên → tính năng
│       │   │   ├── effects.rs    # enum Effect + executor duy nhất
│       │   │   └── jobs.rs       # task mạng: giữ ToolCall, timeout, huỷ job cũ cùng khoá
│       │   ├── conversation.rs   # Idle / Listening / Thinking / Speaking / Offline
│       │   ├── view.rs           # điều hướng, focus, tự quay về, Ambient, "tạm xem", tool server → màn
│       │   ├── alerts.rs         # trạng thái báo toàn màn: hàng đợi, tự tắt, đợi hết lượt nói
│       │   ├── ui_state.rs       # UiState, UserInput
│       │   ├── audio.rs          # AudioCommand, AudioEvent (adapter ở apps/hub)
│       │   ├── features/         # mỗi tính năng một thư mục cùng khuôn (xem 3.2)
│       │   │   ├── timers/       # mod.rs (điều phối), rules.rs, store.rs, tools.rs, tests.rs
│       │   │   ├── countdowns/
│       │   │   ├── reminders/
│       │   │   ├── weather/      # vị trí nhà / tạm xem, WMO → icon & nền
│       │   │   ├── calendar/     # cache 60 ngày, syncToken, sự kiện mới tạo
│       │   │   ├── home/         # entity demo, trạng thái cũ
│       │   │   └── photos/       # 100 ảnh / 200 MB, ghim, xoá cũ nhất
│       │   ├── selector.rs       # chọn mục theo nhãn → Found / Ambiguous / NotFound
│       │   ├── lunar.rs          # âm–dương lịch Việt Nam (UTC+7)
│       │   ├── ports.rs          # Clock, WeatherSource, CalendarSource, HomeSource, ImageFetcher, System
│       │   ├── sources/          # adapter thật của port (chạy được cả Mac và Pi)
│       │   │   ├── open_meteo.rs       # forecast, air quality, geocoding
│       │   │   ├── google_calendar.rs  # events.list/insert, OAuth đổi mã + refresh
│       │   │   ├── home_assistant.rs   # WebSocket API
│       │   │   └── image_fetch.rs      # giới hạn, chặn LAN/localhost, thu nhỏ
│       │   ├── store/
│       │   │   ├── mod.rs        # Store::open(dir) / Store::in_memory(), chạy migration
│       │   │   └── migrations/   # 0001_init.sql, … (method theo tính năng ở features/*/store.rs)
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
│   │   │   └── bridge/           # mod.rs đọc watch<UiState>; <screen>.rs: apply cho từng màn;
│   │   │                         # callback Slint → UserInput
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

Một module sâu: toàn bộ Voice Protocol nằm sau hai channel giới hạn.

```rust
pub fn spawn(config: watch::Receiver<Option<VoiceConfig>>)   // None = chưa cấu hình
    -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>);

pub struct VoiceConfig {
    pub ota_url: Url,
    pub device: DeviceIdentity,   // device_id, client_id
    pub tools: Vec<ToolDef>,      // trả lời Device MCP tools/list
}

pub struct DeviceIdentity { pub device_id: String, pub client_id: String }
pub struct ToolDef { pub name: String, pub description: String, pub input_schema: serde_json::Value }
pub struct OpusFrame(pub Vec<u8>);

pub enum VoiceCommand {
    Connect,                      // thử lại OTA khi đang Enrolling
    ListenStart,                  // listen:start, mode realtime
    ListenStop,
    Abort,                        // abort phiên hiện tại
    Audio(OpusFrame),             // bỏ khi chưa Ready
    DirectSpeech { request_id: String, text: String },
}

pub enum VoiceEvent {
    Connection(ConnectionState),  // Unconfigured | Connecting | Ready | Offline { retry_in } | Enrolling
    Activation { code: String },  // mã activation từ OTA
    Stt { text: String },
    TtsStart { origin: TtsOrigin },
    TtsText { text: String },      // sentence_start
    TtsAudio(OpusFrame),           // binary Opus downlink
    TtsStop { reason: TtsStopReason },
    DirectSpeechRejected { request_id: String, reason: DirectSpeechError },
    ServerTool { name: String, state: ToolState },
    ToolCall(ToolCall),
}

pub enum ConnectionState { Unconfigured, Connecting, Ready, Offline { retry_in: Duration }, Enrolling }
pub enum TtsOrigin { Agent, ClientDirect { request_id: String } }
pub enum TtsStopReason { Completed, Interrupted }
pub enum DirectSpeechError {
    Offline, UnsupportedFeature, InvalidText, InvalidRequestId, SessionBusy,
    RateLimited, TtsUnavailable, CapacityReached, ShuttingDown, Other(String),
}
pub enum ToolState { Started, Completed, Failed }

pub struct ToolCall { pub name: String, pub arguments: serde_json::Value }
impl ToolCall { pub fn reply(self, result: Result<serde_json::Value, ToolError>); }
pub struct ToolError { pub code: i32, pub message: String, pub data: Option<serde_json::Value> }
```

Features `mcp`, `tool_status`, `direct_tts` và `aec` luôn bật trong ClientHello; chúng không nằm trong `VoiceConfig`. Command channel có capacity 64, event channel có capacity 256. Khi event channel đầy, `TtsAudio` bị drop trước để reader WebSocket tiếp tục; các sự kiện khác đợi chỗ trống.

**Giấu bên trong:** `reconnect.rs` sở hữu task client, hai channel, backoff (trần 60 s sau jitter, chỉ reset khi kết nối giữ Ready ≥ 30 s) và reconnect khi config đổi, kèm log nguyên nhân (`connection::Failure`); `ota.rs` làm OTA discovery; `connection.rs` thực hiện WebSocket, xác thực và ClientHello/ServerHello trong giới hạn `Timeouts`; `connection/voice.rs` chạy vòng Ready: ánh xạ command, text/binary frame, Ping định kỳ + idle timeout và lifecycle TTS; `protocol.rs` giữ toàn bộ wire format (hello, frame uplink, parse downlink) private; `mcp.rs` xử lý Device MCP, phân trang `tools/list`, gọi tool (tối đa 16 đang chạy) và reply theo JSON-RPC id số hoặc chuỗi. `ToolCall` bị drop chưa `reply` → gửi lỗi JSON-RPC nội bộ. Socket đóng trong lượt TTS → phát đúng một `TtsStop { reason: Interrupted }` trước `Connection(Offline)`; `tts stop` từ server, kể cả sau `Abort`, phát `Completed`. Đổi `VoiceConfig` chỉ reconnect khi giá trị khác.

**Enrollment:** vela-server phải cấu hình `devices.enrollment.transport = "ota"`. Khi OTA trả `activation`, client phát `Enrolling` và mã, rồi chờ `Connect`, cấu hình thay đổi, hoặc mã hết hạn theo `activation.timeout_ms` trước khi gọi OTA lại. Nếu response có cả `websocket` và `activation`, client ưu tiên WebSocket. Enrollment qua WebSocket không được hỗ trợ.

**Bất biến qua API công khai:**

1. Mỗi `TtsStart` được đóng bởi một `TtsStop`; mất socket giữa lượt phát `Interrupted` trước `Offline`.
2. Trước `Ready`, `Audio`, `ListenStart`, `ListenStop` và `Abort` bị bỏ; `DirectSpeech` nhận `DirectSpeechRejected { reason: Offline }`.
3. `ToolCall` bị drop trước reply gửi lỗi nội bộ; socket đã đóng khiến reply thành no-op.
4. Sự kiện theo đúng thứ tự frame server gửi.
5. Channel giới hạn 64 command và 256 event; audio bị drop trước khi làm nghẽn reader.
6. Chỉ giá trị `VoiceConfig` thay đổi mới làm kết nối lại.
7. Task client và hai channel sống tới khi các sender config đóng; drop event receiver cũng dừng task. `None` → không kết nối, phát `Connection(Unconfigured)`; `Some → None` đóng socket theo bất biến 1. `Unconfigured` xử lý command như trước `Ready`.

Integration test dùng server WebSocket giả tại `tests/`; test backoff dùng tokio time paused.

### 3.2 `hub-core`

#### Interface bên ngoài

Interface bên dưới là thiết kế đích. Hiện `run(config, store, ports, io)` xử lý voice settings, hội thoại và các tool hẹn giờ. `ConfigStore` giữ cấu hình voice; `Store::open(dir)` hoặc `Store::in_memory()` mở SQLite; `tool_definitions()` cung cấp schema cho bốn tool hẹn giờ. Báo hết giờ thuộc ticket 06.

```rust
pub async fn run(config: ConfigStore, store: Store, ports: Ports, io: Io) -> anyhow::Result<()>;

pub struct Io {
    pub voice_config_tx: watch::Sender<Option<VoiceConfig>>, // hub-core dẫn từ ConfigStore + tool_definitions()
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

pub fn tool_definitions() -> Vec<ToolDef>;   // hub-core đưa vào VoiceConfig
```

Gần như mọi hành vi của sản phẩm — luồng voice, 21 tool `hub_*`, điều hướng, báo hết giờ, Reminder, cache offline, Setup web — test được qua đúng interface này: gửi `VoiceEvent`/`AudioEvent`/`UserInput`, tiến clock, rồi kiểm `VoiceCommand`/`AudioCommand` đã gửi và `UiState`.

`run` tự spawn Setup web và actor. Làm mới dữ liệu (thời tiết 30 phút, lịch 5 phút) chạy trên tick của actor qua `Effect::Spawn`; HA đẩy event qua `HomeSource::subscribe()`.

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
    TtsPlaybackFinished,
}
```

Biên độ mic và năng lượng TTS không đi qua `hub-core`: audio ghi thẳng vào `watch<Levels>` mà UI đọc. Hiện tại adapter desktop chỉ tiêu thụ lệnh, chưa thu hoặc phát âm thanh; `UiState.voice` chứa trạng thái hội thoại, transcript và phản hồi.

#### Module bên trong

Actor (`runtime/`) nhận mọi input, gọi các module dưới đây, rồi thực thi `Vec<Effect>` chúng trả về trong `runtime/effects.rs` — nơi duy nhất gửi channel, ghi `Store`, spawn job, reply tool và phát `UiState` mới (`send_if_modified`). Mỗi module có seam nội bộ riêng để unit test, nhưng không phải interface của crate.

Hai lớp:

- **Luật** (`features/*/rules.rs`, `lunar`, `selector`): hàm thuần nhận `now`, trả kết quả nghiệp vụ (`Result<TimerId, TimerError>`, `Vec<DueTimer>`…). Không biết `Effect`.
- **Điều phối** (`conversation`, `view`, `alerts`, `features/*/mod.rs`): nhận input + `now`, trả `Vec<Effect>`. `features/<x>/mod.rs` là chỗ duy nhất dịch kết quả luật sang `Effect`.

| Module | Giữ gì | Interface chính |
|---|---|---|
| `conversation` | Trạng thái voice, chờ im lặng 10 s sau TTS, mic tắt | `on(input, now) -> Vec<Effect>`; `turn_active()` |
| `view` | Màn hiện tại, focus, tự về màn trung tâm sau ~60 s, Ambient, "tạm xem", tool phía server → màn | `open(screen, focus)`, `on_user(..)`, `tick(now)`, `screen_for_server_tool(name)` |
| `runtime::jobs` | Task mạng đang chạy, `ToolCall` đang chờ | `Effect::Spawn { job, work, reply, timeout }` → `Internal::JobDone(id, result)`; hết giờ → reply lỗi; job mới cùng khoá huỷ job cũ |
| `alerts` | Hàng đợi trạng thái báo (hết giờ, Reminder): lần lượt, tự tắt 2 phút / 60 s, đợi hết lượt nói | `push(alert)`, `dismiss()`, `tick(now, turn_active)` |
| `features::timers` | ≤ 4 hẹn giờ, giờ kết thúc tuyệt đối, tạm dừng | `start`, `adjust`, `cancel`, `due(now)` |
| `features::countdowns` | ≤ 8 mốc, `none` / `yearly` / `yearly_lunar` | `create`, `remove`, `next_occurrence(now)` |
| `features::reminders` | `none` / `daily` / `weekly`, đã nhắc, đã lỡ (< 1 giờ thì nhắc một lần) | `create`, `cancel`, `snooze`, `due(now)` |
| `features::weather` | Vị trí nhà, thành phố tạm xem (giữ ~30 phút), WMO → icon/nền | `snapshot(focus)`, `apply(forecast)` |
| `features::calendar` | Cache ~60 ngày, `syncToken`, sự kiện mới tạo | `events(range)`, `apply(delta)` |
| `features::photos` | Chỉ mục ảnh, ghim, chính sách xoá | `add`, `pin`, `evict()` |
| `selector` | Chọn mục theo nhãn (dùng chung cho hẹn giờ, mốc, Reminder) | `select(items, query) -> Found / Ambiguous(Vec) / NotFound` |
| `lunar` | Thuật toán âm lịch Việt Nam (Hồ Ngọc Đức), múi UTC+7 | `to_solar(lunar_date)`, `to_lunar(date)` |
| `store` | SQLite + thư mục ảnh ([ADR 0012](adr/0012-sqlite-for-local-data.md)) | `open`/`in_memory` + migration; method theo tính năng ở `features/*/store.rs`, mỗi thao tác một transaction |
| `config` | File TOML 0600, thông báo khi đổi | `get()`, `update(f)`, `watch()` |

Luật tính năng là **logic thuần nhận `now`, trả kết quả**; actor lo ghi `Store` và phát hiệu ứng. Vì vậy test một luật (vd. Reminder `weekly` qua Chủ nhật) không cần actor.

**Khuôn thư mục tính năng.** Thêm hoặc sửa một tính năng chỉ chạm thư mục của nó, cộng một dòng định tuyến trong `runtime/tools.rs`:

```
features/<x>/
  mod.rs     # điều phối: on_tool, on_job, tick, snapshot → Vec<Effect>
  rules.rs   # luật thuần
  store.rs   # impl Store { <x>_* }
  tools.rs   # struct tham số (serde + schemars), definitions(), parse()
  tests.rs
```

Không có enum `ToolRequest` chung: `runtime/tools.rs` định tuyến theo tiền tố tên (`hub_timer_*` → `timers`), `tool_definitions()` nối `definitions()` của từng tính năng. Không có trait `Feature`: các tính năng khác nhau nhiều (mạng, push, file), runtime gọi qua `match`; chỉ rút trait khi đã có 3–4 tính năng thật cùng hình dạng.

#### Seam bên ngoài (port)

| Port | Adapter thật | Adapter khác |
|---|---|---|
| `Clock` | `SystemClock` (hub-core) | Clock test theo tokio time paused |
| `WeatherSource` (request/response) | `sources::open_meteo` | Fake trong `tests/support` |
| `CalendarSource` | `sources::google_calendar` | Fake |
| `HomeSource` (`subscribe() -> Receiver<HomeEvent>` + `call_service`; adapter tự reconnect) | `sources::home_assistant` | Fake |
| `ImageFetcher` | `sources::image_fetch` | Fake |
| `System` (độ sáng, Wi-Fi, AP setup, khởi động lại/tắt máy) | `apps/hub/system/pi.rs` | `system/desktop.rs`, fake |

Không phải seam (một adapter): `Store` (test dùng `Store::in_memory()`), `ConfigStore` (test dùng thư mục tạm), `setup_web`, OAuth Google (nằm trong `sources::google_calendar`, Setup web gọi trực tiếp).

### 3.3 `apps/hub`

`main.rs` chỉ nối dây:

1. Đọc thư mục dữ liệu, mở `ConfigStore` và `Store`.
2. `voice_client::spawn(voice_config_rx)` đúng một lần; hub-core phát `VoiceConfig` (từ `ConfigStore` + `tool_definitions()`) qua `Io.voice_config_tx`. Không spawn lại, không relay.
3. `audio::spawn(...)` trên thread riêng.
4. Dựng `Ports` (adapter `sources::*` của hub-core, `system` theo feature).
5. Chạy tokio runtime ở thread phụ với `hub_core::run(...)`.
6. Chạy `ui::run_ui(ui_rx, levels_rx, input_tx)` trên main thread (Slint `linuxkms` cần main thread).

**UI bridge:** `bridge/mod.rs` đọc `watch<UiState>` và qua `slint::invoke_from_event_loop` gọi `bridge/<screen>.rs::apply(&app, &state.<screen>)` cho từng màn; callback Slint đổi sang `UserInput`. Logic không nằm ở đây; nếu bridge cần một nhánh `if` theo nghiệp vụ thì nhánh đó thuộc `hub-core`.

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
  → runtime/tools: định tuyến theo tiền tố → features/<x>/tools::parse (lỗi tham số → reply lỗi ngay)
  → features/<x>::on_tool → Vec<Effect>: view.open(screen, focus), Persist, Ui, Reply
  → phần cần mạng (calendar_create, weather location khác, show_image): Effect::Spawn kèm ToolCall,
    runtime/jobs chờ → Internal::JobDone → features/<x>::on_job → hoàn tất trạng thái → Reply
    (quá timeout → jobs reply lỗi)
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

Actor tick 1 Hz cho mọi thứ theo giờ (hẹn giờ, Reminder, tự quay về, chữ số đếm ngược, làm mới thời tiết/lịch); vòng tròn hẹn giờ mượt nhờ Slint `animate` tuyến tính 1 s giữa hai tick. Tick chỉ là nhịp: luật luôn so sánh với giờ tuyệt đối từ `Clock`, không đếm tick.

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
| Luật (`lunar`, `selector`, `features/*/rules.rs`) | Unit test, truyền `now`, kiểm kết quả nghiệp vụ | `cargo test -p hub-core` |
| Điều phối (`conversation`, `view`, `alerts`, `features/*/mod.rs`) | Unit test, truyền input + `now`, kiểm `Vec<Effect>` | `cargo test -p hub-core` |
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

**View hiện có:** `UiState.view` chứa `Screen` và `Focus`; `UserInput::Open`, `Back`, `Activity` điều hướng và đặt lại hạn 60 giây. Tool timer thành công mở màn Hẹn giờ và timer mới được focus theo ID. Bảng ánh xạ tool server và chính sách tự vào Ambient chưa được triển khai.
