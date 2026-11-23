# 10 — Không nuốt im lặng request MCP và kết quả TTS

Status: done
Spec: [../spec.md](../spec.md)
Nguồn: review nhánh feat/voice-client (2026-11-23)

## Việc

- `mcp.rs`: `id` chấp nhận cả số và chuỗi (giữ nguyên `Value` để echo lại). Có `id` nhưng `tools/call` thiếu `params.name` → trả JSON-RPC `-32602`. Không có `id` (notification) mới được bỏ qua.
- `reply_rx`: bỏ phần `id` không dùng, hoặc dùng nó thật — chọn một, không giữ cả hai.
- `connection/voice.rs` `parse_text_event`: `tts result` `rejected` thiếu `reason` → `reason: Other`; thiếu `request_id` → log/ghi rõ hành vi trong spec. `tts start` `source=client_direct` thiếu `request_id` không âm thầm thành `Agent`.
- Giới hạn số `tools/call` đang chạy song song và dùng kênh reply có giới hạn.

## Nghiệm thu

- [x] Test `tools/call` thiếu name → nhận `-32602`; id chuỗi được echo đúng.
- [x] Test `rejected` thiếu reason → caller nhận sự kiện rejected.
- [x] Test vượt giới hạn in-flight → server nhận lỗi, không spawn thêm task.

## Comments

- 2026-11-23: `id` số hoặc chuỗi được echo nguyên; không có `id` là notification, không trả lời; có `id` mà thiếu `method` → `-32600`. Giới hạn `MAX_IN_FLIGHT_TOOL_CALLS = 16` bằng semaphore; vượt quá → `-32000`. Kênh reply có giới hạn 16, permit giữ tới khi reply vào hàng đợi. `reply_rx` bỏ `id`. `rejected` thiếu `reason` → `Other("")`; thiếu `request_id` → bỏ qua kèm `log::warn` (không correlate được). `tts start` `client_direct` thiếu `request_id` → `Agent` kèm `log::warn`. Test: `src/mcp_edge_tests.rs`, `src/connection/voice_tests.rs`.
