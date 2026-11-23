# Spec: hub-core

Status: needs-triage

## Mục tiêu

Xây `crates/hub-core` thành module sâu chứa **toàn bộ hành vi** của hub, theo [docs/codebase.md §3.2](../../docs/codebase.md), ADR 0001 và 0002. Interface của crate là một hàm `run` cùng các channel. Gần như mọi hành vi đều test được qua interface đó trên Mac: gửi input, tiến clock (tokio time paused), rồi kiểm command đã gửi ra và `UiState`.

Spec này sửa thiết kế ở `codebase.md` tại năm điểm, ghi ở mục "Thay đổi so với codebase.md".

## Hiện trạng

- `hub-core` mới là bộ khung: `run(ports, io)` chỉ tick đồng hồ vào `UiState.clock`. `tests/support::TestClock` đã có.
- Hành vi voice settings (task voice-client, ánh xạ `ConnectionState` → nhãn, mã kích hoạt, `config.toml`) đang nằm trong `apps/hub/src/voice_settings/`. Theo ADR 0001, phần này phải chuyển sang hub-core.

## Interface đích

```rust
pub async fn run(config: ConfigStore, store: Store, ports: Ports, io: Io) -> anyhow::Result<()>;

pub struct Io {
    pub voice_config_tx: watch::Sender<Option<VoiceConfig>>, // None = chưa cấu hình, voice-client chưa chạy
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

1. **hub-core sở hữu `VoiceConfig`.** `Io` có `voice_config_tx`. hub-core dẫn `VoiceConfig` từ `ConfigStore` và `tool_definitions()`. `apps/hub` chỉ chuyển giá trị tới `voice_client::spawn` (spawn khi `Some`, dừng khi `None`). Vì vậy luồng "lưu OTA URL → kết nối lại" test được trong hub-core.
2. **Actor theo mô hình Effect.** Mọi module bên trong nhận input + `now` và trả `Vec<Effect>`. `runtime/effects.rs` là nơi duy nhất thực thi hiệu ứng: gửi channel, ghi `Store`, spawn task mạng, reply tool, cập nhật `UiState`. `runtime/` chia thành `mod.rs` (vòng `select!`), `voice.rs`, `audio.rs`, `input.rs`, `tools.rs`, `tick.rs`, `effects.rs`. Seam nội bộ "input → effects" cho phép unit test luật mà không cần channel.
3. **Bất biến thời gian.** `Clock` của test phải tính từ `tokio::time::Instant` + mốc cố định (đã làm ở `TestClock`). Logic không đọc giờ hệ thống; tick 1 Hz dùng `tokio::time::interval`.
4. **`Store` chia theo tính năng.** `store/mod.rs` mở kết nối và chạy migration; `store/timers.rs`, `store/reminders.rs`… mỗi file một nhóm method. Vẫn không có trait (chỉ một adapter); test dùng `Store::in_memory()`.
5. **Đồng bộ giờ.** `System::time_synced() -> bool` (Pi đọc trạng thái NTP; desktop luôn `true`). `alerts` giữ lại mục quá hạn, không báo và không đánh dấu "Đã lỡ" khi giờ chưa đồng bộ.

## Lát dọc (ticket)

| # | Ticket | Phụ thuộc |
|---|---|---|
| 01 | Khung runtime: Effect, executor, harness test | — |
| 02 | ConfigStore + chuyển voice settings vào hub-core | 01 |
| 03 | Conversation + kênh audio | 01 |
| 04 | Store + khung tool `hub_*` + hẹn giờ | 01 |
| 05 | View: điều hướng, focus, tự về màn trung tâm | 04 |
| 06 | Alerts + đồng bộ giờ | 03, 05 |
| 07 | Selector, mốc đếm ngược, âm lịch | 04 |
| 08 | Reminder | 06, 07 |
| 09 | Thời tiết (Open-Meteo) | 05 |
| 10 | Lịch Google | 05 |
| 11 | Home Assistant | 05 |
| 12 | Ảnh | 05 |
| 13 | Setup web | 02 |

Mỗi ticket là một lát đi xuyên interface `run()`, có test trong `crates/hub-core/tests/`. Phần Slint cho từng màn hình thuộc `apps/hub` và chỉ là bridge.

## Ngoài phạm vi

- Adapter audio thật (`cpal`, Opus, openWakeWord), UI Slint, adapter `System` cho Pi: thuộc `apps/hub`.
- Đổi độ phân giải lúc chạy (câu hỏi mở §7 codebase.md).

## Quy ước

- Tuân theo AGENTS.md: file < 300 dòng, test ở file riêng, không có `mod tests {}` inline.
- Mỗi ticket cập nhật `docs/codebase.md` nếu interface thay đổi.
