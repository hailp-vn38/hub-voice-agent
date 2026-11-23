# 02 — ConfigStore + chuyển voice settings vào hub-core

Status: ready-for-agent
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

- `src/config.rs`: `ConfigStore::open(dir)` đọc/ghi `<data>/config.toml` quyền 0600; giữ nguyên định dạng `[voice]` hiện có (device_id, client_id, ota_url) để không mất dữ liệu đã lưu. API: `get()`, `update(f)`, `watch()`.
- Chuyển logic từ `apps/hub/src/voice_settings/` vào hub-core: sinh UUID lần đầu, kiểm URL, ánh xạ `ConnectionState` → nhãn, lưu mã kích hoạt.
- `Io.voice_config_tx: watch::Sender<Option<VoiceConfig>>`; `run` phát `Some` khi có OTA URL, kèm `tool_definitions()` (tạm rỗng).
- `UserInput::SaveVoiceUrl(String)`, `UserInput::VoiceConnect`. `UiState.voice_settings { ota_url, device_id, status_label, activation, error, enrolling }`.
- `apps/hub`: xoá `voice_settings/`; `main.rs` chỉ spawn voice-client theo `voice_config` rx; bridge Slint đọc `UiState` và gửi `UserInput`.

## Nghiệm thu

- [ ] Test hub-core: `SaveVoiceUrl` hợp lệ → `voice_config_tx` đổi, file config lưu URL; khởi động lại `ConfigStore` giữ URL, device_id, client_id.
- [ ] URL sai → `UiState.voice_settings.error`, config không đổi.
- [ ] `VoiceEvent::Connection(Enrolling)` + `Activation{code}` → `UiState` có mã; `VoiceConnect` → `VoiceCommand::Connect`; `Ready` → mã biến mất.
- [ ] Các test cũ của `apps/hub/tests/voice_settings.rs` được chuyển hoặc thay bằng test tương đương ở hub-core.
- [ ] Màn Settings chạy như trước (kiểm tay trên Mac).
