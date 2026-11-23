# Spec: voice-client

Status: needs-info

## Mục tiêu

Hoàn thiện crate `crates/voice-client`: toàn bộ Voice Protocol tới vela-server nằm sau một hàm `spawn` và hai channel, đúng mô tả ở [docs/codebase.md §3.1](../../docs/codebase.md). Spec này có sửa interface ở một số chỗ (xem "Thay đổi so với codebase.md").

Xong spec này thì:

- hub-core (hoặc test) gọi `spawn(config)` là có kết nối Ready tới vela-server, gồm cả lần đầu chưa Enrollment.
- Gửi `ListenStart` + frame Opus thì nhận `Stt`, `TtsStart…TtsStop` và audio downlink.
- Server gọi được tool `hub_*` qua Device MCP; hub-core trả lời qua `ToolCall::reply`.
- Hub đọc được văn bản bằng giọng server qua `DirectSpeech`.
- Mất mạng thì tự reconnect có backoff; đổi `VoiceConfig` thì kết nối lại.
- Mọi hành vi trên đều có test trên Mac với server WS giả, không cần Pi hay vela-server thật.

## Nguồn giao thức (vela-server)

| Chủ đề | Tài liệu |
|---|---|
| Admission, handshake, ingress, thứ tự outbound, đóng socket | `docs/flows/01-websocket.md` |
| OTA + Enrollment Session qua WebSocket | ADR 0073, ADR 0074 |
| Device MCP (`initialize`, `tools/list`, `tools/call`) | `docs/flows/07-device-mcp.md` |
| Tool status `type:"tool"` | `docs/flows/01-websocket.md` § Tool execution status |
| `direct_speech` | ADR 0161, `.scratch/client-direct-speech/` |
| Wire type thật | `crates/vela-server/src/protocol/` |

Đường dẫn tính từ repo `AI_Agent_Voice_rust`. Khi tài liệu và code server khác nhau, code server là chuẩn. Ghi lại chỗ khác nhau vào `## Comments` của ticket.

## Interface

```rust
pub fn spawn(config: watch::Receiver<VoiceConfig>)
    -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>);

pub struct VoiceConfig {
    pub ota_url: Url,                 // điểm vào duy nhất; URL WS + token lấy từ OTA
    pub device: DeviceIdentity,       // device_id, client_id (header `device-id`, `client-id`)
    pub tools: Vec<ToolDef>,          // trả lời tools/list
}

pub enum VoiceCommand {
    ListenStart,                      // listen:start, mode realtime
    ListenStop,
    Abort,
    Audio(OpusFrame),                 // uplink 16 kHz mono 60 ms
    DirectSpeech { request_id: String, text: String },
}

pub enum VoiceEvent {
    Connection(ConnectionState),      // Connecting | Ready | Offline { retry_in } | Enrolling
    Activation { code: String },      // chỉ trong Enrollment Session
    Stt { text: String },
    TtsStart { origin: TtsOrigin },   // Agent | ClientDirect { request_id }
    TtsText { text: String },         // sentence_start
    TtsAudio(OpusFrame),              // downlink 24 kHz mono 60 ms
    TtsStop { reason: TtsStopReason },// Completed | Interrupted
    DirectSpeechRejected { request_id: String, reason: DirectSpeechError },
    ServerTool { name: String, state: ToolState },
    ToolCall(ToolCall),
}
```

Features trong ClientHello cố định bên trong crate: `mcp`, `tool_status`, `direct_tts`, `aec` = `true`. Hub không có lý do tắt chúng, nên không đưa ra interface.

### Bất biến của interface

1. **Lượt TTS luôn đóng.** Mỗi `TtsStart` có đúng một `TtsStop` theo sau. Nếu socket đứt giữa lượt, voice-client phát `TtsStop { reason: Interrupted }` *trước* `Connection(Offline)`.
2. **Không có lệnh nào bị nuốt im lặng, trừ audio.** Khi chưa `Ready`:
   - `Audio`: bỏ, không báo (frame cũ không còn giá trị).
   - `ListenStart`, `ListenStop`, `Abort`: bỏ. hub-core đã biết trạng thái kết nối qua `Connection`.
   - `DirectSpeech`: trả `DirectSpeechRejected { reason: Offline }`.
3. **`ToolCall` luôn được trả lời.** Drop mà chưa `reply` → voice-client gửi lỗi JSON-RPC nội bộ. Socket đứt trước khi reply → `reply` thành no-op.
4. **Thứ tự sự kiện giữ đúng thứ tự frame server gửi.**
5. **Channel có giới hạn.** `VoiceCommand` capacity 64. `VoiceEvent` capacity 256. Khi channel sự kiện đầy, `TtsAudio` bị drop trước để không chặn reader WS. Các sự kiện khác chờ chỗ trống.
6. **Đổi `VoiceConfig` thì kết nối lại**, trừ khi giá trị mới bằng giá trị cũ.

## Giấu bên trong

OTA HTTP, WebSocket + header xác thực, ClientHello/ServerHello, Enrollment Session, reconnect backoff, `session_id`, frame text vs binary, JSON-RPC của Device MCP (trả lời `initialize`, phân trang `tools/list`, đối chiếu id cho `tools/call`), ánh xạ `tts state:"result"` sang `DirectSpeechRejected`.

Cấu trúc file theo [codebase.md §2](../../docs/codebase.md): `protocol.rs`, `connection.rs`, `enrollment.rs`, `mcp.rs`, `tests/support/fake_server.rs`.

## Thay đổi so với codebase.md

| codebase.md | Spec này | Lý do |
|---|---|---|
| `Enrolled(Credentials)` → hub-core ghi config | Bỏ | Enrollment của vela-server diễn ra trong WebSocket (ADR 0074): sau khi claim, server đóng `1000`, client kết nối lại bằng cùng Device ID. Không có credential mới nào phải lưu. |
| `ServerConfig { URL OTA; URL WS + token }` | `ota_url` | Mỗi lần kết nối đều gọi OTA. OTA trả URL WS + token theo trạng thái Device. Không cache trong config. |
| `features` trong `VoiceConfig` | Cố định trong crate | Không có caller nào cần đổi. |
| `PlaybackAck { .. }` | Bỏ | Playback ACK (`reminder.played`, ADR 0090) chỉ dành cho Reminder phía server, mà hub không dùng. Direct Speech kết thúc bằng `tts state:"result"`. |
| `TtsStop` | `TtsStop { reason }` + bất biến 1 | hub-core không bị kẹt ở Speaking khi mất mạng. |
| — | `Abort`, `Connection(Enrolling)` | Barge-in bằng chạm cần lệnh `abort`; UI cần phân biệt "đang chờ kích hoạt" với "offline". |
| — | `DirectSpeechRejected { reason: Offline }` | hub-core chỉ có một đường fallback (chuông + chữ). |

Ticket cuối cập nhật docs/codebase.md theo bảng này.

## Ngoài phạm vi

- Encode/decode Opus, thiết bị âm thanh (thuộc `apps/hub`).
- Logic hội thoại: khi nào gửi `ListenStart`, timeout im lặng 10 s (thuộc `hub-core::conversation`).
- Lưu `VoiceConfig` và Setup web.
- `listen:detect`, Device Push Channel, Reminder phía server, `turn_status` (ADR 0004 và concept.md).
- Tách crate giao thức dùng chung với server.

## Quyết định

- **Async:** tokio. WS: `tokio-tungstenite` + rustls. HTTP OTA: `reqwest` + rustls. JSON: `serde`/`serde_json`. Khai báo trong `[workspace.dependencies]`.
- **Backoff:** 1 s nhân đôi, jitter ±20 %, rồi chặn trần 60 s (trần áp sau jitter). Chỉ reset về 1 s khi kết nối đã `Ready` liên tục ≥ 30 s; đứt sớm hơn thì tiếp tục tăng. Đóng `1000` sau Enrollment thì kết nối lại ngay, không backoff.
- **Mode listen:** luôn `realtime` (ADR 0004 của ai-hub).
- **Không retry `tools/call`.** Mỗi request trả lời đúng một lần.
- **Test:** server WS giả + OTA HTTP giả chạy trên `127.0.0.1:0`; backoff test bằng `tokio::time::pause`. Kịch bản theo tài liệu server ở trên.

## Tiêu chí nghiệm thu

1. Test qua `scripts/agent-test.sh` của crate `voice-client` đều qua; clippy `-D warnings` và fmt qua.
2. Mỗi bất biến 1–6 có ít nhất một test đi qua interface công khai.
3. Không type nào trong `protocol.rs` là `pub` ra ngoài crate.
4. `cargo tree -p voice-client` không có Slint, cpal hay opus.
5. docs/codebase.md §3.1 khớp interface cuối.

## Tickets

| # | Ticket | Blocked by | Status |
|---|---|---|---|
| 01 | [Handshake tới Ready qua server giả](issues/01-handshake-ready.md) | — | ready-for-human |
| 02 | [OTA lấy URL WS và token](issues/02-ota-discovery.md) | 01 | ready-for-human |
| 03 | [Enrollment Session và mã kích hoạt](issues/03-enrollment-session.md) | 02 | needs-info |
| 04 | [Reconnect có backoff, đổi config](issues/04-reconnect-backoff.md) | 02 | ready-for-human |
| 05 | [Listen, audio uplink, STT/TTS downlink](issues/05-listen-and-tts.md) | 01 | ready-for-human |
| 06 | [Device MCP và tool status](issues/06-device-mcp.md) | 01 | ready-for-human |
| 07 | [Direct Speech](issues/07-direct-speech.md) | 05 | ready-for-human |
| 08 | [Đóng lượt TTS khi mất kết nối, cập nhật codebase.md](issues/08-invariants-and-docs.md) | 04, 05, 06, 07 | ready-for-human |
