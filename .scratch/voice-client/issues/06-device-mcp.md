# 06 — Device MCP và tool status

Status: ready-for-agent
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

- [ ] Test: server giả gửi `initialize` rồi `tools/list` → nhận đủ tool trong config.
- [ ] Test: `tools/call` → `ToolCall` có đúng `name`, `arguments`; `reply(Ok)` → server nhận `result` cùng `id`.
- [ ] Test: drop `ToolCall` → server nhận lỗi cùng `id`.
- [ ] Test: reply sau khi socket cũ đứt và đã reconnect → socket mới không nhận gì.
- [ ] Test: `type:"tool"` `started`/`completed` → hai `ServerTool`.
