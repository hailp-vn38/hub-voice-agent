# 06 — Device MCP và tool status

Status: ready-for-human
Blocked by: 01
Spec: [../spec.md](../spec.md)

## Việc

Theo `docs/flows/07-device-mcp.md` của vela-server: server là JSON-RPC client, hub là MCP server. Message bọc trong `{"type":"mcp","session_id":…,"payload":{jsonrpc…}}`.

- `mcp.rs`: trả lời `initialize` (đối chiếu field yêu cầu với code server), `tools/list` từ `VoiceConfig.tools` (tên, description, `inputSchema`; hỗ trợ `cursor`/`nextCursor` nếu server gửi), `tools/call` → `VoiceEvent::ToolCall`.
- `ToolCall::reply(Ok(v))` → JSON-RPC `result`; `Err(ToolError)` → `error` hoặc `result.isError` theo cách server đọc (kiểm code server).
- Bất biến 3: `ToolCall` drop chưa reply → trả lỗi nội bộ; socket đã đứt → `reply` no-op. Reply sau khi đã reconnect không được gửi vào socket mới.
- Method lạ → lỗi JSON-RPC `-32601`.
- `type:"tool"` → `ServerTool { name, state }`.

## Nghiệm thu

- [x] Test: server giả gửi `initialize` rồi `tools/list` → nhận đủ tool trong config, gồm phân trang.
- [x] Test: `tools/call` → `ToolCall` có đúng `name`, `arguments`; `reply(Ok)` → server nhận `result` cùng `id`.
- [x] Test: drop `ToolCall` → server nhận lỗi cùng `id`.
- [x] Test: reply sau khi socket cũ đứt và đã reconnect → socket mới không nhận gì.
- [x] Test: `type:"tool"` `started`/`completed` → hai `ServerTool`.

## Comments

- Đã đối chiếu actor, `device_mcp::wire`, `protocol::downlink` và flow 07: không có khác biệt wire cần sửa. Server gửi JSON-RPC numeric ID trong wrapper `type:"mcp"` và chỉ cần result từ `initialize` trước khi yêu cầu `tools/list`; client trả trang tối đa 100 tool với cursor offset. Server đọc lỗi `tools/call` như JSON-RPC error nhưng chuyển mọi code/message thành lỗi Device chung.
- Đã chạy `cargo test -p voice-client`, `cargo clippy -p voice-client --all-targets -- -D warnings`, `cargo fmt --all -- --check` và `scripts/test.sh -p voice-client`.
