# 01 — Khung runtime: Effect, executor, harness test

Status: ready-for-agent
Blocked by: —
Spec: [../spec.md](../spec.md)

## Việc

- Chuyển actor sang `src/runtime/`: `mod.rs` (vòng `select!` qua tick và channel), `effects.rs` (`enum Effect` + executor duy nhất), `tick.rs` (đồng hồ hiện có).
- `Effect` ban đầu: `Ui(fn/patch cập nhật UiState)`, `Voice(VoiceCommand)`, `Audio(AudioCommand)`. Các ticket sau thêm biến thể.
- Thêm vào `Io`: `voice_tx`, `voice_rx`, `input_rx` (chưa xử lý gì, chỉ nối vào `select!`). `run` kết thúc khi `ui_tx` đóng như hiện nay.
- `tests/support`: harness `Hub::start()` dựng `run()` với mọi channel, `TestClock`, và trả các đầu channel cho test.

## Nghiệm thu

- [ ] Test `clock` hiện có vẫn qua, viết lại bằng harness.
- [ ] Unit test executor: một `Effect::Voice` → command xuất hiện trên `voice_tx`.
- [ ] `apps/hub` vẫn build và hiển thị đồng hồ (`cargo build -p hub`).
- [ ] Hành vi không đổi; code di chuyển và tách ở commit riêng.
