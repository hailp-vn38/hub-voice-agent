# 04 — Voice dock: Orb

Status: ready-for-human
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Concept voice-first" mục 1 (Orb), §4 "Trạng thái", "Hiệu ứng theo trạng thái"

## Việc

- `contract/voice.slint`:
  - `export enum VoiceState { idle, listening, thinking, speaking, offline }`.
  - `global Voice { in property <VoiceState> state; in property <bool> mic-muted; in property <int> wake-serial; in property <int> tool-serial; in property <int> error-serial; callback orb-tapped(); }`.
  - `global VoiceLevels { in property <float> mic; in property <float> tts; }` (0..1, ~30 Hz).
- `components/orb.slint`: vài hình tròn chồng nhau, chỉ scale/opacity/rotation.

| Trạng thái | Hiệu ứng |
|---|---|
| idle / Ambient | Thở scale ±3%, chu kỳ ~4 s, nhỏ và mờ |
| `wake-serial` đổi | Bật sáng, lớn lên ~250 ms, sóng lan một vòng về phía màn |
| listening | Co giãn theo `VoiceLevels.mic`; vòng ngoài mờ lan theo tiếng nói |
| thinking | Xoay chậm, chấm nhỏ quay quanh |
| `tool-serial` đổi | Bắn một hạt về phía màn |
| speaking | Đập theo `VoiceLevels.tts` |
| offline / `error-serial` | Xám, rung nhẹ một lần |
| `mic-muted` | Xám có gạch |

- Chạm Orb → `Voice.orb-tapped()`. Vùng chạm ≥ `Theme.touch-min`.
- Neo toạ độ tâm Orb trong một global `Dock { out property <length> orb-x; out property <length> orb-y; }` để ticket 06 dùng làm điểm xuất phát.
- Demo: `VoiceLevels` giả bằng sóng sin + nhiễu khi listening/speaking.

## Demo

Kịch bản: "Wake", "Nghe", "Nghĩ", "Nói", "Tool chạy", "Lỗi", "Offline", "Tắt mic", "Về Idle".

## Nghiệm thu

- [ ] Mỗi trạng thái trong bảng nhìn khác nhau rõ, chuyển mượt giữa các trạng thái.
- [ ] Đổi `wake-serial` hai lần liên tiếp chạy hiệu ứng hai lần; ghi lại cùng giá trị không chạy.
- [ ] Orb đứng yên ở mọi màn.
- [ ] Đo fps trên Pi ở listening và speaking (ready-for-human).

## Map với hub-core

Map: chờ hub-core #03 (conversation); levels từ adapter audio ở `apps/hub`

| Hợp đồng | UiState / UserInput |
|---|---|
| `Voice.state` | |
| `Voice.mic-muted` | |
| `Voice.wake-serial` | |
| `Voice.tool-serial` | |
| `Voice.error-serial` | |
| `Voice.orb-tapped()` | |
| `VoiceLevels.mic` / `.tts` | (watch `Levels` từ audio, không qua hub-core) |

## Answer

Thêm `contract/voice.slint` (Voice, VoiceLevels), `components/orb.slint` (Orb + global Dock, neo tâm bằng absolute-position), gắn Orb vào hàng dock của Shell. Hiệu ứng wake/tool/error dựa trên `changed` serial + `animation-tick()`. Demo: `demo/levels.rs` (sóng sin + nhiễu, có test) và 9 kịch bản. Chưa xem bằng mắt/đo fps trên Pi (ready-for-human).

## Comments

- 2026-11-23 (review hub-ui): Orb có `disabled` (xám, không chạm được), dùng ở màn lần đầu khởi động; Orb thu nhỏ và mờ ở Ambient. Vị trí: góc phải dưới. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
