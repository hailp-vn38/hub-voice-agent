# Spec: hub-core

Status: needs-triage

## Mục tiêu

Xây `crates/hub-core` thành module sâu chứa **toàn bộ hành vi** của hub, theo [docs/codebase.md §3.2](../../docs/codebase.md), ADR 0001 và 0002. Interface của crate là một hàm `run` cùng các channel. Gần như mọi hành vi đều test được qua interface đó trên Mac: gửi input, tiến clock (tokio time paused), rồi kiểm command đã gửi ra và `UiState`.

Spec này sửa thiết kế ở `codebase.md` tại các điểm ghi trong mục "Thay đổi so với codebase.md".

## Hiện trạng

- `hub-core` mới là bộ khung: `run(ports, io)` chỉ tick đồng hồ vào `UiState.clock`. `tests/support::TestClock` đã có.
- Hành vi voice settings (task voice-client, ánh xạ `ConnectionState` → nhãn, mã kích hoạt, `config.toml`) đang nằm trong `apps/hub/src/voice_settings/`. Theo ADR 0001, phần này phải chuyển sang hub-core.

## Interface đích

```rust
pub async fn run(config: ConfigStore, store: Store, ports: Ports, io: Io) -> anyhow::Result<()>;

pub struct Io {
    pub voice_config_tx: watch::Sender<Option<VoiceConfig>>, // None = chưa cấu hình
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

pub fn tool_definitions() -> Vec<ToolDef>;
```

Field của `Ports` và `Io` được thêm dần theo từng ticket; mỗi ticket chỉ thêm phần nó cần.

## Thay đổi so với codebase.md

1. **hub-core sở hữu `VoiceConfig`; channel voice ổn định.** `Io` có `voice_config_tx`. hub-core dẫn `VoiceConfig` từ `ConfigStore` và `tool_definitions()`. voice-client nhận `watch<Option<VoiceConfig>>` và sống suốt vòng đời app ([voice-client #14](../voice-client/issues/14-optional-config.md)), nên `voice_tx`/`voice_rx` không bao giờ đổi. `apps/hub` gọi `voice_client::spawn` đúng một lần, không có relay. Luồng "lưu OTA URL → kết nối lại" test được trong hub-core.
2. **Actor theo mô hình Effect, hai lớp.**
   - **Luật** (`features/*/rules.rs`, `lunar`, `selector`): hàm thuần nhận `now`, trả kết quả nghiệp vụ (`Result<TimerId, TimerError>`, `Vec<DueTimer>`…). Không biết `Effect`.
   - **Điều phối** (`conversation`, `view`, `alerts`, `features/*/mod.rs`): nhận input + `now`, trả `Vec<Effect>`. Mỗi `features/<x>/mod.rs` là chỗ duy nhất dịch kết quả luật sang `Effect`.
   - `runtime/effects.rs` là nơi duy nhất thực thi hiệu ứng: gửi channel, ghi `Store`, spawn job, reply tool, cập nhật `UiState`. `runtime/` chia thành `mod.rs` (vòng `select!`), `voice.rs`, `audio.rs`, `input.rs`, `tools.rs` (định tuyến), `tick.rs`, `effects.rs`, `jobs.rs`.
3. **Bất biến thời gian.** `Clock` của test phải tính từ `tokio::time::Instant` + mốc cố định (đã làm ở `TestClock`). Logic không đọc giờ hệ thống; tick 1 Hz dùng `tokio::time::interval` và chỉ là nhịp: luật luôn so sánh với giờ tuyệt đối từ `Clock`, không đếm tick.
4. **Tổ chức theo tính năng, không theo tầng.** Mỗi tính năng là một thư mục; thêm hoặc sửa một tính năng chỉ chạm thư mục đó, cộng một dòng định tuyến:

   ```
   features/timers/
     mod.rs     # điều phối: on_tool, tick, snapshot → Vec<Effect>
     rules.rs   # luật thuần
     store.rs   # impl Store { timers_* } — Store vẫn một struct, không trait
     tools.rs   # struct tham số (serde + schemars), definitions(), parse()
     tests.rs
   ```

   - `store/mod.rs` chỉ mở kết nối và chạy migration; method theo tính năng nằm trong `features/<x>/store.rs`.
   - Không có `tools/args.rs` hay enum `ToolRequest` chung. `runtime/tools.rs` định tuyến theo tiền tố tên (`hub_timer_*` → `timers`); `tool_definitions()` nối `definitions()` của từng tính năng.
   - **Không có trait `Feature`** lúc này: các tính năng khác nhau nhiều (mạng, push, file). Runtime gọi qua `match`. Chỉ rút trait khi đã có 3–4 tính năng thật có cùng hình dạng.
5. **Job cho việc bất đồng bộ.** `runtime/jobs.rs` sở hữu mọi task mạng mà tool hoặc làm mới dữ liệu cần:

   ```rust
   Effect::Spawn { job: JobId, work: BoxFuture<'static, JobResult>, reply: Option<ToolCall>, timeout: Duration }
   // xong → actor nhận Internal::JobDone(JobId, JobResult) → feature.on_job(..) → Vec<Effect>
   ```

   `jobs` giữ `ToolCall` trong lúc chờ, áp timeout (hết hạn → reply lỗi), và huỷ job cũ khi tính năng phát job mới cùng khoá (vd. hỏi thời tiết Đà Nẵng rồi Huế ngay). Tính năng không tự spawn task.
6. **Port theo kiểu nguồn.**
   - `WeatherSource`, `CalendarSource`, `ImageFetcher`: request/response (`async fn`). Lịch làm mới (30 phút, 5 phút) chạy trên tick của actor và phát `Effect::Spawn`; `run` không spawn vòng làm mới riêng. Test: tiến clock → fake được gọi.
   - `HomeSource`: `subscribe() -> mpsc::Receiver<HomeEvent>` (`Connected`, `State`, `Disconnected`) + `async fn call_service(..)`. Adapter tự lo reconnect; actor chỉ thấy event.
7. **`Store` chia theo tính năng.** Vẫn không có trait (chỉ một adapter); test dùng `Store::in_memory()`. Xem điểm 4 về vị trí file.
8. **Đồng bộ giờ.** `System::time_synced() -> bool` (Pi đọc trạng thái NTP; desktop luôn `true`). `alerts` giữ lại mục quá hạn, không báo và không đánh dấu "Đã lỡ" khi giờ chưa đồng bộ.
9. **Bỏ module `server_tools`.** Bảng tên tool phía server → màn là một `match` trong `view` (`view::screen_for_server_tool`).
10. **UI bridge theo màn.** `apps/hub/src/ui/bridge/mod.rs` đọc `watch<UiState>` rồi gọi `bridge/<screen>.rs::apply(&app, &state.<screen>)`, khớp một màn một file `.slint`. Bridge không chứa nhánh nghiệp vụ.

## Lát dọc (ticket)

| # | Ticket | Phụ thuộc |
|---|---|---|
| 01 | Khung runtime: Effect, executor, jobs, harness test | — |
| 02 | ConfigStore + chuyển voice settings vào hub-core | 01, voice-client #14 |
| 03 | Conversation + kênh audio | 01 |
| 04 | Store + mẫu thư mục tính năng + hẹn giờ | 01 |
| 05 | View: điều hướng, focus, tự về màn trung tâm | 04 |
| 06 | Alerts + đồng bộ giờ | 03, 05 |
| 07 | Selector, mốc đếm ngược, âm lịch | 04 |
| 08 | Reminder | 06, 07 |
| 09 | Thời tiết (Open-Meteo) | 05 |
| 10 | Lịch Google | 05 |
| 11 | Home Assistant | 05 |
| 12 | Ảnh | 05 |
| 13 | Setup web | 02 |

Mỗi ticket là một lát đi xuyên interface `run()`, có test trong `crates/hub-core/tests/`. Phần Slint cho từng màn hình thuộc `apps/hub` và chỉ là bridge. Ticket 04 dựng `features/timers/` làm khuôn; các ticket 07–12 theo đúng khuôn đó.

## Ngoài phạm vi

- Adapter audio thật (`cpal`, Opus, openWakeWord), UI Slint, adapter `System` cho Pi: thuộc `apps/hub`.
- Đổi độ phân giải lúc chạy (câu hỏi mở §7 codebase.md).

## Quy ước

- Tuân theo AGENTS.md: file < 300 dòng, test ở file riêng, không có `mod tests {}` inline.
- Mỗi ticket cập nhật `docs/codebase.md` nếu interface thay đổi.
