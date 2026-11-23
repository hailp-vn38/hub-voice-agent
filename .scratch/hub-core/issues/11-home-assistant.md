# 11 — Home Assistant

Status: ready-for-agent
Blocked by: 05
Spec: [../spec.md](../spec.md)

## Việc

- Port `HomeSource`: `subscribe() -> mpsc::Receiver<HomeEvent>` (`Connected`, `State`, `Disconnected`) + `async fn call_service(..)`. Adapter `sources/home_assistant.rs` (WebSocket API: auth, subscribe state, call service) tự kết nối lại; actor chỉ thấy event.
- `src/features/home/` (khuôn ticket 04): entity hiển thị, đánh dấu trạng thái cũ khi mất kết nối.
- Tool bật/tắt/xem thiết bị, chọn entity bằng `selector`.

## Nghiệm thu

- [ ] Adapter test với fixture message HA.
- [ ] Test qua `run()`: tool bật đèn → fake nhận call service, `UiState.home` cập nhật khi fake đẩy state mới.
- [ ] Mất kết nối → entity đánh dấu cũ, tool reply lỗi rõ.
