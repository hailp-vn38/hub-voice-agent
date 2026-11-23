# 05 — Chữ phản hồi, dải bước tool, gợi ý câu nói

Status: ready-for-human
Blocked by: 04
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Concept voice-first" mục 2, 3, 5 (gợi ý câu nói); "Điều hướng" (gợi ý theo màn)

## Việc

- `contract/response.slint`:
  - `struct Word { text: string, emphasis: bool }`; `export enum Speaker { user, agent }`.
  - `global Response { in property <int> line-serial; in property <Speaker> speaker; in property <[Word]> words; in property <bool> fading; }`. Mỗi câu mới (`stt` hoặc `sentence_start`) là một `line-serial` mới; `words` đã tách từ và đánh dấu phần nhấn.
- `components/response-text.slint` (kinetic text):
  - Một dòng ngang hàng Orb, căn phải sát Orb, không xuống dòng.
  - Mỗi từ mới trượt vào từ phải, xoay/phóng nhẹ rồi rung tắt (~550 ms, overshoot); nhịp hiện từ cố định do Slint quyết.
  - Từ cũ bị đẩy sang trái và mờ ở mép trái (mask gradient).
  - `user` màu nhạt hơn `agent`; `emphasis` tô màu nhấn.
  - Chỉ animate vài từ mới nhất cùng lúc.
  - `fading` → chữ mờ dần (sau 10 s im lặng, hub-core quyết).
- `contract/tool-trail.slint`: `export enum StepState { running, done, failed }`; `struct ToolStep { key: string, label: string, state: StepState }`; `global ToolTrail { in property <[ToolStep]> steps; }`.
- `components/tool-trail.slint`: dải chấm ngay trên dòng chữ cạnh Orb; chấm mới xuất hiện, `done` xanh, `failed` đỏ.
- **Gợi ý câu nói** khi Idle: dòng chữ cạnh Orb xoay vòng câu mẫu, khác nhau theo màn (trung tâm, menu: câu điều khiển tính năng; màn tính năng: câu liên quan màn). Danh sách câu nằm trong `.slint` theo `Nav.screen`.

## Demo

Kịch bản: "Người dùng nói" (stt), "Agent trả lời" (3 câu, có từ nhấn "7 phút"), "Tool server: lịch" (running → done), "Tool lỗi", "Im lặng → mờ".

## Nghiệm thu

- [ ] Câu dài: từ cũ trôi sang trái và mờ, không xuống dòng, không tràn qua Orb.
- [ ] Phân biệt rõ chữ người dùng và chữ agent; từ nhấn có màu nhấn.
- [ ] Idle ở màn trung tâm, menu, Hẹn giờ cho ba bộ gợi ý khác nhau.
- [ ] Đo fps trên Pi khi đang hiện chữ (ready-for-human).

## Map với hub-core

Map: chờ hub-core #03 (stt, tts text, fading), #05 (ServerTool)

| Hợp đồng | UiState / UserInput |
|---|---|
| `Response.line-serial` | |
| `Response.speaker` | |
| `Response.words` | |
| `Response.fading` | |
| `ToolTrail.steps` | |

## Answer

Da lam: contract/response.slint, contract/tool-trail.slint, components/response-text.slint (chu dong, goi y theo Nav.screen, xoay 5 s), components/tool-trail.slint, gan vao shell.slint canh Orb. Demo: 5 kich ban (src/ui/demo/response.rs, co unit test split_words). Mep trai mo dung overlay gradient (Slint khong co mask). Chua do fps tren Pi (ready-for-human).

## Comments

- 2026-11-23 (review hub-ui): Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
