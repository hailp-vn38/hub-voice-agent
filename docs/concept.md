# AI Hub — Tài liệu ý tưởng

> Hướng dẫn phát triển trên máy Mac: [development.md](development.md).
>
> Trạng thái: **ý tưởng**, chưa triển khai. Cập nhật: 2026-11-24.

## 1. Mục tiêu

Một thiết bị hub AI cá nhân trên máy tính bảng Linux (Debian 13) có màn cảm ứng:

- UI đẹp, có hiệu ứng, chạy toàn màn hình ở chế độ kiosk; bật máy là vào thẳng app.
- Là **Physical Device client** của vela-server: nói chuyện với voice agent qua Voice Protocol.
- Là hub thông tin: đồng hồ, thời tiết, lịch, Reminder, điều khiển nhà.
- Client "dày": có wake word cục bộ và cache offline; vẫn hiển thị được khi mất mạng.

### Ngoài phạm vi bản đầu

- Điều khiển nhà bằng giọng nói (bản đầu chỉ demo bằng chạm).
- Sửa/xoá sự kiện lịch (bản đầu chỉ tạo mới).
- VAD cục bộ (dùng VAD của server).

## 2. Phần cứng

| Thành phần | Lựa chọn |
|---|---|
| Máy | Máy tính bảng Fujitsu Q5010, x86_64, Debian 13 trixie |
| Màn hình | 5 inch cảm ứng, đặt ngang; xuất **1080p** (panel có thể là 2K, scale lên) |
| Mic, loa | Tích hợp trong máy tính bảng |

Máy không có AEC phần cứng. AEC là điều kiện để wake word và barge-in hoạt động khi loa đang phát; cách khử vọng còn phải chọn (ADR 0005).

## 3. Kiến trúc

### Repo

```
ai-hub/
├── crates/voice-client   # chỉ kết nối voice theo mô hình client–server
├── crates/hub-core       # toàn bộ hành vi của hub, không UI/audio/OS
└── apps/hub              # ứng dụng chạy trên máy tính bảng
```

Thiết kế chi tiết từng module: [codebase.md](codebase.md); quyết định đã chốt: [adr/](adr/).

**`voice-client`** — không UI, không đụng thiết bị âm thanh:

- WebSocket tới vela-server, ClientHello/ServerHello, các message Voice Protocol.
- Gửi/nhận frame Opus 60 ms qua channel.
- Reconnect có backoff.
- Test được trên macOS, không cần máy tính bảng.

Type giao thức định nghĩa lại ở client; chỉ tách crate giao thức dùng chung với server khi giao thức đã ổn định.

**`apps/hub`**:

- Slint UI (backend `linuxkms`, renderer GLES).
- Audio I/O, mã hoá/giải mã Opus.
- Wake word **`hey_jarvis`** (model có sẵn của openWakeWord, chạy qua ONNX Runtime). Đổi từ khoá khác để sau khi sản phẩm hoàn thiện.
- Widget: đồng hồ, thời tiết, lịch, Reminder, Home Assistant.
- Cache offline, setting.
- Setup web: HTTP server nhỏ trong cùng process, luôn mở cho LAN.

### Process

```
┌──────────── ai-hub (1 process, tokio) ─────────────┐
│ Slint UI  ◄── UiState (watch channel) ──┐          │
│                                         │          │
│ voice-client (WS) ◄─► SessionLoop ──────┤          │
│ AudioIn → wake word → Opus enc ─┘ │     │          │
│ AudioOut ← Opus dec ◄─────────────┘     │          │
│ Widgets (weather / Calendar / HA) → cache ┘         │
│ Setup web (HTTP, mở cho LAN) → config              │
└────────────────────────────────────────────────────┘
        │ WebSocket                   │ HTTPS / WS
        ▼                             ▼
   vela-server          Google Calendar API, thời tiết, Home Assistant
```

- Audio chạy trên thread riêng, không chặn UI thread.
- UI chỉ đọc `UiState`; mỗi trạng thái voice có animation riêng.

## 4. Luồng voice

### Trạng thái

| Trạng thái | Ý nghĩa | UI |
|---|---|---|
| Idle | WS ở Ready (kết nối, không gửi mic); wake word đang nghe cục bộ | Màn hub bình thường |
| Listening | Đang stream mic lên server | Sóng âm theo biên độ mic |
| Thinking | Server đang ASR/LLM | Pulse |
| Speaking | Đang phát TTS | Waveform theo audio TTS |
| Offline | Mất kết nối server | Badge offline; widget dùng cache |

### Luồng

```
Idle
 └─ wake word (hoặc chạm nút mic)
      → listen:start (realtime) — không gửi listen:detect (xem dưới)
      → stream Opus 60 ms liên tục; server VAD cắt câu → ASR → LLM → TTS
      → người dùng nói chen khi Speaking → server barge-in
 └─ chạm Orb, hoặc đang nghe mà im lặng 5 phút → ngừng gửi mic → Idle
```

- **VAD do server làm.** Client không chạy VAD; chỉ có wake word.
- **Không gửi `listen:detect` khi nghe wake word.** Server coi `text` của `listen:detect` là một lượt nói đã gõ sẵn và chạy LLM ngay ("typed detect"), nên gửi "hey jarvis" sẽ thành một câu hỏi thật. Hub không dùng `listen:detect`.
- **Wake word chạy cục bộ:** `hey_jarvis` ("Hây Gia-vít"). Model là tiếng Anh nên cần chỉnh threshold khi thử với giọng Việt.
- **Ring buffer ngắn** sau wake word để không mất audio trong lúc gửi `listen:start`.
- **Giữ WS ở Ready** khi Idle để tránh độ trễ admission mỗi lần đánh thức.
- **Barge-in:** dùng `realtime` mode (VAD giữ armed qua Speaking). Client khai báo `features.aec=true`; server cần bật `barge_in.enabled=true` và `barge_in.trust_client_aec_feature=true`.
- Wake word chỉ dùng để thoát Idle; trong hội thoại nói chen là ngắt được.
- **Kết thúc hội thoại:** người dùng chạm Orb, hoặc đang nghe (sau wake word hay sau khi TTS kết thúc) mà 5 phút không có lời nói; client ngừng stream mic và về Idle. Thao tác trên hub không ngắt hội thoại. Giới hạn 5 phút để không gửi âm thanh phòng lên server vô thời hạn.

### Kết nối server

- Lần đầu: Enrollment/OTA để lấy URL WebSocket và token; hiển thị mã kích hoạt trên màn hình.
- Không dùng Device Push Channel (MQTT) và Reminder của server: Push Channel chỉ mang Wake Signal cho Reminder server, mà hub tự quản Reminder cục bộ (xem "Màn Reminder").

## 5. Widget và dữ liệu

| Widget | Nguồn | Cache offline |
|---|---|---|
| Đồng hồ | Giờ hệ thống (NTP) | — |
| Thời tiết | Client tự gọi **Open-Meteo** (miễn phí, không cần API key) | Bản gần nhất + thời điểm cập nhật |
| Lịch | **Google Calendar API** (OAuth): đọc và tạo sự kiện | Các sự kiện ~60 ngày tới |
| Reminder | Cục bộ trên hub (tạo bằng giọng nói qua `hub_reminder_*`) | Lưu cục bộ, không cần mạng |
| Nhà (demo) | Home Assistant WebSocket API, long-lived access token | Trạng thái gần nhất, đánh dấu cũ |

### Google Calendar: API + OAuth

OAuth device flow (nhập mã trên điện thoại) **không hỗ trợ scope Calendar**; chỉ cho `openid`/`email`/`profile`, Drive (`drive.file`, `drive.appdata`) và YouTube ([nguồn](https://developers.google.com/identity/protocols/oauth2/limited-input-device)).

- **Đọc và tạo đều qua Google Calendar API** (đã chốt, bỏ iCal): dữ liệu thời gian thực, sự kiện vừa tạo hiện ngay, một nguồn duy nhất.
- **Tạo sự kiện bằng giọng nói làm ở client** qua tool Device MCP `hub_calendar_create` (agent gọi qua kết nối voice).
- Scope: `calendar.events` trên lịch chính (`primary`).
- **Kết nối Google một lần qua Setup web:** device flow không dùng được (không có scope Calendar), và Google không nhận redirect về IP LAN. Cách làm: Setup web mở trang đồng ý của Google (client loại "Desktop app", redirect `http://127.0.0.1:<port>`); sau khi đồng ý, trình duyệt điện thoại chuyển tới một trang lỗi `127.0.0.1` — người dùng chép nguyên URL trên thanh địa chỉ dán vào Setup web; hub đổi mã lấy refresh token và lưu. App Google phải ở trạng thái **published** (ở Testing thì refresh token hết hạn sau 7 ngày); dùng cá nhân sẽ thấy cảnh báo "ứng dụng chưa được xác minh".

### Thời tiết: Open-Meteo

- Miễn phí cho dùng phi thương mại, không cần đăng ký hay API key.
- Toạ độ (vĩ độ/kinh độ) nhập qua Setup web.
- Gọi theo chu kỳ (ví dụ 30 phút); dữ liệu cập nhật theo giờ nên không cần gọi dày hơn.
- Map `weather_code` (WMO) sang icon và animation trên UI.

### Home Assistant (demo)

- Vài thẻ: bật/tắt một đèn, một công tắc, hiển thị một cảm biến.
- Địa chỉ và token nhập qua Setup web; entity demo chọn trên màn hình.
- Điều khiển bằng giọng nói (server gọi HA qua MCP) để sau.

## 6. UI và hiển thị

- **Slint**, backend `linuxkms`, renderer **FemtoVG** (OpenGL ES): vẽ thẳng ra DRM/KMS, không X11/Wayland.
- **1080p** mặc định; setting cho chọn mode KMS khác.
- Layout theo tỉ lệ và `scale-factor`, không pixel cố định; chọn scale theo mode khi khởi động.
- Hiệu ứng dựa trên transform/opacity/gradient; tránh blur toàn màn hình (GPU tích hợp yếu).
- Đo fps thực tế trên máy tính bảng trước khi chốt hiệu ứng.

### Nguyên tắc

- Đọc được từ xa 1–2 m: giờ và nhiệt độ là chữ lớn nhất.
- **Màn trung tâm** là nơi người dùng ở gần như mọi lúc: không tab, không cuộn.
- **Vuốt từ phải sang trái** để vào **Menu tính năng**; vuốt từ trái sang phải để quay về (xem mục Điều hướng).
- Voice là **Voice dock** cố định ở góc phải dưới, luôn hiện; card hub chiếm phần còn lại của màn. Không có màn voice riêng.
- Vùng chạm tối thiểu ~9–10 mm (~80–90 px ở 1080p trên 5 inch); chỉ chạm và vuốt đơn giản.

### Màn hình

| Màn | Nội dung | Vào bằng |
|---|---|---|
| Chính | Thanh trạng thái; đồng hồ + thời tiết; lịch hôm nay; Reminder; 2–4 thẻ nhà; Voice dock ở góc phải dưới | Mặc định |
| Voice dock (góc phải dưới, mọi màn) | Orb, dòng chữ phản hồi chạy sang trái, dải bước tool, thẻ tool | Luôn hiện; kích hoạt bằng wake word, chạm Orb |
| Menu tính năng | Lưới ô tính năng, mỗi ô có thông tin sống | Vuốt phải → trái từ màn trung tâm |
| Thời tiết | Theo giờ, 7 ngày | Menu, chạm card thời tiết, hoặc `hub_open_screen` |
| Lịch | Các sự kiện sắp tới | Menu, chạm card lịch, hoặc `hub_open_screen` |
| Nhà | Toàn bộ thẻ demo | Menu, chạm card nhà, hoặc `hub_open_screen` |
| Hẹn giờ | Các hẹn giờ đang chạy, huỷ bằng chạm | Menu hoặc `hub_open_screen` |
| Ngày đặc biệt | Các mốc, xoá bằng chạm | Menu hoặc `hub_open_screen` |
| Reminder | Reminder sắp tới (cục bộ) | Menu hoặc `hub_open_screen` |
| Ảnh | Các ảnh agent đã gửi gần đây (cache) | Menu hoặc `hub_open_screen` |
| Setting | Chỉ lựa chọn + IP/QR Setup web | Menu hoặc icon bánh răng |
| Ambient | Đồng hồ lớn + nhiệt độ, độ sáng thấp | Không chạm sau N phút hoặc ban đêm; chạm/wake word để thoát |

Bố cục màn chính (ngang 16:9):

```
┌──────────────────────────────────────────────┐
│ thanh trạng thái                             │
│ ┌────────────┐ ┌────────┐ ┌────────┐         │
│ │ đồng hồ    │ │ lịch   │ │ đếm    │  card   │
│ │ thời tiết  │ ├────────┤ ├────────┤  hub    │
│ │            │ │Reminder│ │ nhà    │         │
│ └────────────┘ └────────┘ └────────┘         │
│      ← ← chữ phản hồi rung, chạy từ Orb  (●) │  Voice dock
└──────────────────────────────────────────────┘
```

Card hub dùng lưới phía trên; hàng dưới cùng dành cho Voice dock, Orb neo ở góc phải. Kích thước theo đơn vị logic × `scale-factor`.

### Điều hướng

```
           vuốt ← (phải sang trái)
┌─────────────┐  ───────────▶  ┌──────────────┐   chạm ô   ┌──────────────┐
│ Màn trung   │                │ Menu tính    │ ─────────▶ │ Màn tính năng│
│ tâm         │  ◀───────────  │ năng         │ ◀───────── │ (thời tiết…) │
└─────────────┘  vuốt →        └──────────────┘   vuốt →   └──────────────┘
        ▲                                                        │
        └──────────── hub_open_screen / chạm card ───────────────┘
```

- **Hai tầng ngang:** màn trung tâm ⇄ Menu tính năng. Vuốt phải → trái để vào menu, trái → phải để quay lại; chấm chỉ trang nhỏ ở mép trên cho biết đang ở đâu.
- **Menu tính năng:** lưới 4 × 2 ô lớn — Thời tiết, Lịch, Nhà, Hẹn giờ, Ngày đặc biệt, Reminder, Ảnh, Setting. Mỗi ô có icon, tên và một dòng thông tin sống (vd. "1 đang chạy", "3 sự kiện hôm nay").
- **Màn tính năng:** chạm ô để mở; vuốt trái → phải để về menu. Không lồng sâu hơn một tầng.
- **Đường tắt:** chạm card trên màn trung tâm, hoặc agent gọi `hub_open_screen`, mở thẳng màn tính năng; vuốt về thì quay lại màn trung tâm.
- **Tự quay về:** không có tương tác sau ~60 giây thì trở về màn trung tâm (rồi sang Ambient theo luật cũ).
- **Voice dock không đổi chỗ:** Orb đứng yên ở góc phải dưới khi chuyển màn; chỉ nội dung phía trên trượt. Nói "Hey Jarvis" ở bất kỳ màn nào đều được.
- **Hiệu ứng chuyển:** menu trượt vào từ phải theo ngón tay (theo sát cử chỉ, không chờ nhấc tay); màn trung tâm lùi lại (scale ~0.92, mờ đi) tạo chiều sâu. Nhấc tay quá ~30% chiều rộng hoặc vuốt nhanh thì hoàn tất, ngược lại bật về. ~400 ms, chỉ dùng transform và opacity.
- **Vùng cử chỉ:** vuốt bắt đầu trên Orb không tính là vuốt menu (chạm Orb để nói); vuốt dọc không dùng.
- **Gợi ý câu nói đổi theo màn:** ở menu là các câu điều khiển tính năng ("hẹn giờ 5 phút"); ở màn tính năng là câu liên quan tới màn đó.

### Màn Thời tiết

Bố cục ba cột phía trên Voice dock:

| Vùng | Nội dung |
|---|---|
| Đầu màn | "‹ Thời tiết · {tên vị trí}" bên trái; "Cập nhật HH:MM" bên phải (badge offline nếu dữ liệu cũ) |
| Cột trái — Hiện tại | Icon thời tiết, nhiệt độ lớn, mô tả (vd. "Mưa nhẹ"), cảm giác như, độ ẩm, gió, UV, **chất lượng không khí** (AQI + mức: Tốt / Trung bình / Kém… tô màu theo mức), giờ mọc/lặn |
| Cột giữa — 24 giờ tới | Đường nhiệt độ mượt, nhãn mỗi 6 giờ, điểm cao nhất được đánh dấu; cột mờ phía dưới là khả năng mưa theo giờ |
| Cột phải — 7 ngày | Mỗi hàng: thứ, icon, thanh khoảng nhiệt (thấp → cao) trên cùng một thang cho cả tuần; hàng "Mai" nổi bật |

**Nền động theo thời tiết** (lớp dưới cùng, nhẹ):

| `weather_code` (WMO) | Nền |
|---|---|
| Quang / ít mây | Vầng sáng tròn chậm xoay (ban ngày) hoặc vài chấm sao nhấp nháy (ban đêm) |
| Nhiều mây | 2–3 đám mây trôi chậm, lệch tốc độ tạo chiều sâu |
| Mưa / mưa rào | Mây + ~20–40 hạt mưa rơi xiên |
| Giông | Như mưa + chớp sáng toàn màn ngắn, thưa (vài chục giây một lần) |
| Sương mù | Lớp mờ trôi ngang |

Màu nền ngả theo ngày/đêm dựa trên giờ mọc/lặn. Giới hạn số hạt và chỉ dùng transform/opacity để giữ fps trên máy tính bảng.

**Tương tác chạm:**

- Chạm một điểm trên đường 24 giờ → hiện nhiệt độ, khả năng mưa, gió của giờ đó.
- Chạm một ngày → đường giữa chuyển sang 24 giờ của ngày đó.
- **Không cuộn ngang** trong màn: vuốt ngang đã dành cho điều hướng (vuốt → để về menu).

**Tương tác giọng nói** — màn phản ứng theo câu hỏi:

- "Mai có mưa không?" → hàng "Mai" sáng lên, đường giữa chuyển sang ngày mai, câu trả lời chạy cạnh Orb.
- "Chiều nay mấy độ?" → điểm buổi chiều trên đường 24 giờ được đánh dấu.
- "Thời tiết Đà Lạt thế nào?" → màn chuyển sang Đà Lạt ("tạm xem").
- "Không khí hôm nay thế nào?" → dòng AQI sáng lên.
- "Mở thời tiết" ở bất kỳ màn nào → `hub_open_screen`.

**Dữ liệu:** Open-Meteo, gọi mỗi 30 phút, một request lấy cả `current`, `hourly` (nhiệt độ, khả năng mưa, mã thời tiết, gió) và `daily` (cao/thấp, mã thời tiết, mọc/lặn, UV), 7 ngày. Lưu cache bản gần nhất.

**Tool:** `hub_weather_get` (đã chốt) — xem mục "View theo giọng nói". Agent trả lời bằng đúng dữ liệu hub đang hiển thị.

**Chất lượng không khí:** Open-Meteo Air Quality API (miễn phí) — US AQI và PM2.5, gọi cùng chu kỳ với dự báo.

**Xem thời tiết thành phố khác bằng giọng nói:**

- "Thời tiết Đà Nẵng thế nào?" → agent gọi `hub_weather_get(location = "Đà Nẵng")`.
- Hub tra tên qua **Open-Meteo Geocoding API** (miễn phí, `language=vi`) → lấy toạ độ → tải dự báo + AQI cho nơi đó → mở màn Thời tiết của thành phố đó.
- Đầu màn ghi "‹ Thời tiết · Đà Nẵng" kèm nhãn **"tạm xem"**; nền và dữ liệu đổi theo nơi đó.
- Không lưu thành vị trí nhà: khi rời màn hoặc tự quay về màn trung tâm, màn Thời tiết trở lại vị trí nhà. Giữ dữ liệu thành phố đã xem ~30 phút để hỏi tiếp không phải tải lại.
- Tên trùng nhiều nơi (vd. "Hồng Bàng") → hub trả vài kết quả cho agent; agent hỏi lại người dùng. Không tìm thấy → hub trả lỗi, agent nói lại.
- Đổi vị trí nhà vẫn làm ở Setup web.

### Màn Lịch

Bố cục phía trên Voice dock:

| Vùng | Nội dung |
|---|---|
| Đầu màn | "‹ Lịch · Tháng {N}" (chạm tên tháng → lưới tháng); "Đồng bộ HH:MM" bên phải |
| Dải 7 ngày | Hôm nay → 6 ngày tới, ô lớn dễ chạm; ngày đang chọn tô màu nhấn; chấm nhỏ = số sự kiện (tối đa 3 chấm) |
| Trái — Dòng thời gian ngày đang chọn | Dải "Cả ngày" ở trên; các khối sự kiện theo giờ; **vạch "bây giờ"** màu đỏ nhấp nháy nhẹ; sự kiện đang diễn ra sáng hơn; cuộn dọc được |
| Phải — Sắp tới | 5 sự kiện kế tiếp sau ngày đang chọn: ngày, giờ, tiêu đề; ngày có nhiều sự kiện gộp "+N sự kiện" |

**Lưới tháng:** chạm tên tháng hoặc nói "tháng sau có gì" → lưới tháng thay cho dòng thời gian và danh sách; mỗi ngày có chấm sự kiện; chạm một ngày → quay về dòng thời gian của ngày đó.

**Reminder trong lịch:** Reminder (cục bộ) hiện trên dòng thời gian dạng khối viền đứt nét, khác với sự kiện Google — một nơi xem cả ngày của mình.

**Sự kiện vừa tạo bằng giọng nói:** thẻ sinh ra từ Orb, bay vào đúng ngày trong dải 7 ngày và mục "Sắp tới", nền xanh nhạt, ghi "Mới tạo" (sự kiện đã nằm trên Google Calendar, không phải chờ đồng bộ).

**Tương tác chạm:**

- Chạm ngày trong dải 7 ngày → đổi ngày đang chọn.
- Chạm sự kiện → thẻ chi tiết: tiêu đề, giờ, địa điểm, mô tả (nếu ICS có).
- Cuộn **dọc** trong dòng thời gian được; **không cuộn ngang** (vuốt ngang dành cho điều hướng).

**Tương tác giọng nói** (`hub_calendar_get` với `focus`):

| Câu nói | Màn phản ứng |
|---|---|
| "Hôm nay tôi có lịch gì?" | Chọn hôm nay, cuộn dòng thời gian tới sự kiện kế tiếp |
| "Thứ Sáu có gì?" | Ngày thứ Sáu sáng lên trong dải 7 ngày, dòng thời gian chuyển sang ngày đó |
| "Lịch tiếp theo là gì?" | Sự kiện kế tiếp nhấp sáng ở cả dòng thời gian và "Sắp tới" |
| "Tháng sau có gì?" | Mở lưới tháng sau |
| "Thêm lịch khám răng 9 giờ sáng thứ Ba" | `hub_calendar_create` → thẻ sự kiện sinh ra từ Orb, bay vào thứ Ba |

**Dữ liệu lịch:** Google Calendar API `events.list` (`singleEvents=true` để API tự mở rộng sự kiện lặp), lấy hôm nay → ~60 ngày tới; làm mới mỗi 5 phút bằng `syncToken` (chỉ lấy phần thay đổi) và ngay sau mỗi lần tạo sự kiện. Không dùng push notification của Google (cần webhook HTTPS công khai). Mất mạng → dùng cache, đầu màn ghi giờ đồng bộ cuối.

### Màn Hẹn giờ

Hẹn giờ chạy và lưu **cục bộ** trên hub (còn nguyên khi mất mạng hoặc khởi động lại; thời điểm kết thúc lưu theo giờ tuyệt đối).

**Bố cục** phía trên Voice dock:

| Vùng | Nội dung |
|---|---|
| Đầu màn | "‹ Hẹn giờ · N đang chạy" |
| Giữa — Các hẹn giờ | Mỗi hẹn giờ một thẻ: vòng tròn cạn dần, số còn lại lớn ở giữa, nhãn, "X phút · xong lúc HH:MM", nút tạm dừng / +1 phút / huỷ. 1 hẹn giờ → một vòng lớn chiếm giữa màn; 2 → hai cột; 3–4 → lưới 2×2; tối đa 4 |
| Dưới — Đặt nhanh | Nút 1 / 3 / 5 / 10 / 15 phút (không cần gõ); nhãn chỉ đặt bằng giọng nói |
| Trống | Không có hẹn giờ → vòng trống lớn + gợi ý "Hey Jarvis, hẹn giờ 5 phút" |

**Hiệu ứng:**

- Hẹn giờ mới sinh ra từ Orb, nảy vào chỗ trống trong lưới; các thẻ khác dời chỗ mượt.
- Vòng cạn dần liên tục (không giật theo giây); chữ số đổi mỗi giây.
- **10 giây cuối:** vòng và số chuyển màu ấm, vòng nhịp theo từng giây.
- Tạm dừng: vòng mờ đi, số nhấp nháy chậm.
- Huỷ: vòng co lại về tâm rồi biến mất.

**Hết giờ** (ở bất kỳ màn nào, kể cả Ambient):

- Màn hình sáng lên, chuyển sang trạng thái báo toàn màn: nền ấm, icon chuông với vòng lan theo nhịp, "{nhãn} · hết giờ".
- Chuông lặp lại qua loa (cần AEC để wake word vẫn nghe được khi chuông kêu).
- Tắt: **chạm bất kỳ đâu**, hoặc "Hey Jarvis, tắt" / "thêm 1 phút".
- Tự tắt sau 2 phút nếu không ai phản hồi; thẻ hẹn giờ ghi "Đã hết giờ lúc HH:MM" cho tới khi được xoá.
- Nhiều hẹn giờ cùng hết: báo lần lượt theo thứ tự.
- Đang hội thoại voice thì chuông vẫn kêu nhưng nhỏ hơn, không cắt ngang câu trả lời.

**Trên màn trung tâm:** khi có hẹn giờ chạy, màn trung tâm dành một ô nhỏ cho hẹn giờ sắp xong nhất (vòng + số); chạm ô mở màn Hẹn giờ.

**Tương tác giọng nói:**

| Câu nói | Tool | Màn phản ứng |
|---|---|---|
| "Hẹn giờ luộc trứng 7 phút" | `hub_timer_start` | Mở màn Hẹn giờ, thẻ mới sinh ra từ Orb |
| "Còn bao lâu?" / "Trứng còn bao lâu?" | `hub_timer_list` | Mở màn, thẻ được hỏi sáng lên |
| "Huỷ hẹn giờ trứng" | `hub_timer_cancel` | Thẻ co lại, biến mất |
| "Thêm 2 phút cho trứng" / "Bớt 1 phút" | `hub_timer_adjust` | Vòng nở ra / co lại tới thời lượng mới |
| "Tạm dừng" / "Tiếp tục" | `hub_timer_adjust` | Vòng mờ đi / sáng lại |
| "Tắt" (khi đang báo) | `hub_timer_cancel` | Tắt chuông, đóng trạng thái báo |

Không có nhãn thì hub tự đặt nhãn theo thời lượng ("Hẹn giờ 5 phút"); câu nói mơ hồ khi có nhiều hẹn giờ → agent hỏi lại "hẹn giờ nào?".

### Màn Ngày đặc biệt

Đếm số ngày tới một ngày/giờ (tên trong code: `Countdown`), chạy và lưu **cục bộ** trên hub; tối đa ~8 mốc.

**Bố cục** phía trên Voice dock:

| Vùng | Nội dung |
|---|---|
| Đầu màn | "‹ Ngày đặc biệt · N mốc" |
| Trái — Mốc chính | Mốc gần nhất (hoặc mốc vừa được hỏi/chạm): nhãn, số ngày rất lớn, ngày đích (kèm ngày âm lịch nếu là mốc âm lịch), thanh tiến độ "X% chặng đường" tính từ lúc tạo hoặc từ lần lặp trước |
| Phải — Các mốc khác | Sắp theo mốc gần nhất: nhãn, số ngày, ngày đích, "hằng năm" nếu lặp |
| Trống | Gợi ý "Hey Jarvis, đếm ngày đến Tết" |

**Đơn vị hiển thị theo khoảng cách:**

- ≥ 1 ngày: "N ngày".
- < 24 giờ: chuyển sang **HH:MM:SS chạy trực tiếp**, màu ấm.
- Đúng ngày: trạng thái "Hôm nay là {nhãn}".

**Lặp lại** (tham số `repeat` của `hub_countdown_create`):

| `repeat` | Ví dụ | Hành vi sau khi tới mốc |
|---|---|---|
| `none` | Chuyến đi, hạn nộp hồ sơ | Ghi "Đã qua", tự ẩn sau 1 ngày |
| `yearly` | Sinh nhật, kỷ niệm | Tự chuyển sang năm sau |
| `yearly_lunar` | Tết, giỗ, Rằm | Hub tự đổi ngày âm lịch → dương lịch cho năm tiếp theo |

Mốc âm lịch cần bộ chuyển đổi âm–dương lịch Việt Nam chạy trên hub, để agent chỉ cần nói "mùng 1 tháng Giêng" mà không phải tự tính ngày dương mỗi năm.

**Hiệu ứng:**

- Mốc mới sinh ra từ Orb và bay vào danh sách; nếu là mốc gần nhất thì thay chỗ mốc chính, mốc cũ trượt sang danh sách.
- Qua nửa đêm: số ngày lật (flip) xuống một.
- Chạm một mốc ở danh sách → mốc đó phóng lên vị trí mốc chính.
- **Ngày đến mốc:** khi mở màn hoặc lần đầu màn hình sáng trong ngày đó, pháo giấy (~30 mảnh) bắn ra từ Orb, chữ "Hôm nay là {nhãn}"; màn trung tâm hiện ô mốc đó suốt ngày.
- Xoá: ô mờ dần, các ô khác dồn lên.

**Chạm:** chạm mốc → đưa lên lớn; chạm mốc chính → thẻ chi tiết có nút **Xoá**. Không cần nhấn giữ.

**Tương tác giọng nói:**

| Câu nói | Tool | Màn phản ứng |
|---|---|---|
| "Lưu sinh nhật vợ ngày 12/3" | `hub_countdown_create` (`repeat: yearly`) | Mốc mới sinh ra từ Orb |
| "Đếm ngày đến Tết" | `hub_countdown_create` (`repeat: yearly_lunar`) | Như trên, kèm ngày âm lịch |
| "Còn bao nhiêu ngày đến Tết?" | `hub_countdown_list` (`focus`: nhãn) | Mốc Tết lên vị trí chính |
| "Xoá ngày chuyến đi" | `hub_countdown_remove` | Ô mờ dần |

Hỏi về mốc chưa tạo ("còn bao lâu đến Trung thu?") → agent trả lời trực tiếp và có thể hỏi "có muốn lưu ngày này không?".

**Chọn tool giữa Hẹn giờ, Ngày đặc biệt và Reminder** (mô tả tool ghi các luật này để agent chọn đúng):

| Câu nói | Tool |
|---|---|
| Một khoảng thời gian: "hẹn giờ 7 phút", "đếm ngược 10 phút" | `hub_timer_start` |
| "Nhắc tôi …", "báo thức …", giờ trong ngày, lặp hằng ngày/tuần | `hub_reminder_create` (kể cả "nhắc tôi sau 20 phút …": `at` = bây giờ + 20 phút) |
| Số ngày tới một ngày: "đếm ngày đến Tết", "lưu sinh nhật" | `hub_countdown_create` |
| Lịch hẹn trên Google Calendar: "thêm lịch họp …" | `hub_calendar_create` |

### Màn Reminder

Reminder **chạy và lưu cục bộ trên hub**, giống Hẹn giờ và Ngày đặc biệt. Hub **không dùng** Reminder của vela-server (không khai báo feature `reminders`, không dùng Wake Signal). Nhờ vậy hub có đầy đủ nội dung, vẫn nhắc khi mất mạng.

**Bố cục** phía trên Voice dock:

| Vùng | Nội dung |
|---|---|
| Đầu màn | "‹ Reminder · N sắp tới" |
| Trái — Kế tiếp | Reminder gần nhất: giờ lớn, "còn X phút/giờ", nội dung, "lặp hằng ngày" nếu có |
| Phải — Dòng thời gian | Các Reminder theo ngày (Hôm nay / Ngày mai / …), mỗi dòng: giờ + nội dung + icon lặp |
| Dưới — Đã nhắc gần đây | 3–5 Reminder đã nhắc trong 24 giờ qua; mục bị lỡ ghi "Đã lỡ" |
| Trống | "Hey Jarvis, nhắc tôi uống thuốc lúc 9 giờ tối" |

**Lặp lại** (tham số `repeat`): `none`, `daily`, `weekly` (kèm các thứ trong tuần). Mỗi lần nhắc xong, Reminder lặp tự lên lịch lần kế tiếp.

**Lúc Reminder đến hạn** (ở bất kỳ màn nào, kể cả Ambient):

1. Màn hình sáng lên, chuyển sang **trạng thái nhắc toàn màn** (cùng kiểu báo hết giờ, khác màu): icon chuông, giờ, nội dung chạy ra từ Orb.
2. Phát tiếng chuông, sau đó **đọc nội dung bằng giọng của server** (xem dưới).
3. Trạng thái nhắc giữ ~60 giây rồi thu vào "Đã nhắc gần đây". Chạm để đóng sớm; "Hey Jarvis, nhắc lại sau 10 phút" để hoãn.
4. Nhiều Reminder cùng giờ: nhắc lần lượt. Đang hội thoại voice: đợi lượt nói hiện tại xong rồi mới nhắc.
5. Hub tắt hoặc khởi động lại lúc đến hạn: khi bật lên, Reminder đã quá hạn hiện ở "Đã nhắc gần đây" với nhãn "Đã lỡ" và nhắc một lần nếu chưa quá 1 giờ.

**Đọc nội dung bằng giọng server — message mới ở vela-server (đã chốt):** hub gửi văn bản, server chuyển thẳng sang TTS rồi gửi audio về hub, **không qua ASR/LLM**, không ghi vào lịch sử hội thoại.

```json
{"type":"direct_speech","session_id":"...","request_id":"rem-42","text":"Đến giờ uống thuốc"}
```

- Thiết kế ghi ở vela-server: ADR 0161 (`docs/adr/0161-client-direct-speech-message.md`, Proposed) và spec `.scratch/client-direct-speech/`.
- Kết quả luôn gắn `request_id`: frame `tts` kèm `source: "client_direct"`, hoặc `tts` `state: "result"`, `outcome: "rejected"`, `reason` (vd. `session_busy`, `rate_limited`). Text tối đa 1.024 ký tự; giới hạn 1 giây mỗi Device, dùng chung với Admin `/speak`.
- Client bật bằng `features.direct_tts=true` trong ClientHello (feature này đã có, hiện chỉ bật metadata Direct Speech trên `tts` start/stop).
- Server dùng lại đường Direct Speech có sẵn (ADR 0088: `SessionActor → SpeechOutput → TTS Worker → Opus → WebSocket Writer`); enum `SpeechOrigin` đã có biến thể `ClientDirect` nhưng chưa có message nào kích hoạt.
- Hub nhận `tts` start/stop như lượt nói bình thường, hiển thị chữ chạy từ Orb, gửi Playback ACK.
- Đang có lượt nói khác: server từ chối (theo ADR 0088, Direct TTS của client bận thì reject) → hub đợi rồi gửi lại.
- Mất mạng / server không sẵn sàng: chỉ chuông + hiện chữ.

**Tương tác giọng nói:**

| Câu nói | Tool | Màn phản ứng |
|---|---|---|
| "Nhắc tôi uống thuốc lúc 9 giờ tối" | `hub_reminder_create` | Mở màn Reminder, mục mới sinh ra từ Orb vào dòng thời gian |
| "Mỗi sáng 7 giờ nhắc tôi tập thể dục" | `hub_reminder_create` (`repeat: daily`) | Như trên, có icon lặp |
| "Tối nay tôi có nhắc gì?" | `hub_reminder_list` | Mở màn, các mục tối nay sáng lên |
| "Huỷ nhắc uống thuốc" | `hub_reminder_cancel` | Mục mờ dần |
| "Nhắc lại sau 10 phút" (khi đang nhắc) | `hub_reminder_snooze` | Trạng thái nhắc thu lại, mục dời giờ trên dòng thời gian |

**Trên màn khác:** ô Reminder trên màn trung tâm và khối viền đứt nét trên dòng thời gian màn Lịch dùng cùng dữ liệu cục bộ này.

**Phía server:** agent của hub phải dùng `hub_reminder_*` thay cho tool Reminder builtin của server — chặn tool Reminder builtin cho Device này (tool allowlist) hoặc dặn trong prompt/Template; nếu không, agent có thể tạo Reminder ở server mà hub không hiển thị.

### Màn Ảnh

Ảnh do agent gửi qua `hub_show_image`. Hub lưu **cache cục bộ** để xem lại.

**Lưới ảnh** (vào từ Menu hoặc "cho xem lại ảnh"):

| Vùng | Nội dung |
|---|---|
| Đầu màn | "‹ Ảnh · N ảnh"; "M đã giữ" |
| Lưới | 4 × 2 ô thumbnail mới nhất trước; mỗi ô có chú thích + thời điểm; cuộn **dọc** để xem cũ hơn |
| Ô lỗi | Ảnh không tải được → icon ảnh hỏng + "Không tải được" |
| Trống | "Hey Jarvis, cho tôi xem ảnh vịnh Hạ Long" |

**Xem ảnh** (khi agent gửi, hoặc chạm thumbnail):

- Ảnh toàn màn, zoom/trượt rất chậm (Ken Burns); chú thích + tên miền nguồn ở góc trái dưới; Voice dock vẫn ở góc phải dưới.
- **Nhiều ảnh một lần** (tối đa 6): chấm chỉ số ở mép trên; tự chuyển ảnh mỗi ~6 giây; **chạm mép phải/trái** để sang ảnh kế/trước. Không dùng vuốt ngang để chuyển ảnh — vuốt ngang vẫn giữ nghĩa điều hướng như mọi màn.
- Chạm giữa ảnh → hiện/ẩn chú thích. Vuốt → (trái sang phải) → về lưới ảnh.
- Ảnh do agent gửi tự đóng khi hội thoại kết thúc và ~60 giây không tương tác (về màn trung tâm); ảnh mở từ lưới thì ở lại đến khi người dùng rời.

**Hiệu ứng tải:** thẻ khung chờ sinh ra từ Orb, nở dần thành toàn màn khi ảnh tải xong (crossfade); lỗi → thẻ rung nhẹ, ghi "Không tải được ảnh".

**Giữ ảnh:** chạm biểu tượng ghim trên ảnh (hoặc "giữ ảnh này lại") → ảnh không bị xoá khi cache đầy.

**Cache:**

- Giữ tối đa ~100 ảnh hoặc ~200 MB (cái nào tới trước); xoá ảnh cũ nhất chưa ghim.
- Lưu ảnh đã thu nhỏ về độ phân giải màn hình + thumbnail; không lưu bản gốc.
- Xoá: thẻ chi tiết ảnh có nút Xoá; Setting có "Xoá toàn bộ ảnh".

**Tương tác giọng nói:**

| Câu nói | Tool | Màn phản ứng |
|---|---|---|
| "Cho tôi xem ảnh Đà Lạt" | (tool tìm ảnh phía server) → `hub_show_image` | Ảnh toàn màn; nhiều ảnh thì chạy lần lượt |
| "Ảnh tiếp" / "Ảnh trước" | `hub_image_control` | Chuyển ảnh |
| "Giữ ảnh này lại" | `hub_image_control` | Ghim nhảy lên ảnh |
| "Cho xem lại ảnh lúc nãy" | `hub_image_list` → `hub_image_control` | Mở ảnh được nhắc tới từ cache |

`hub_show_image` nhận **danh sách URL** (1–6) kèm chú thích từng ảnh.

### Hiệu ứng theo trạng thái

| Trạng thái | Hiệu ứng | Ghi chú hiệu năng |
|---|---|---|
| Idle | Đồng hồ chuyển số nhẹ, nền đổi màu chậm theo giờ trong ngày | Cập nhật mỗi phút |
| Listening | Thanh sóng theo biên độ mic | 7–20 hình chữ nhật |
| Thinking | Vòng pulse (scale + opacity) | Chỉ transform |
| Speaking | Waveform theo audio TTS, chữ trả lời hiện dần | ~30 lần/giây |
| Chuyển màn | Slide/fade 200–300 ms | Không blur; dùng lớp màu tối mờ |

### Phong cách

- Nền tối mặc định; theme sáng để sau.
- Một màu nhấn cho trạng thái voice, một màu ấm cho thiết bị đang bật.
- Font sans hỗ trợ tốt tiếng Việt (Inter hoặc Be Vietnam Pro), nhúng vào app; hai độ đậm.
- Bộ icon outline thống nhất (Tabler hoặc Lucide, xuất SVG).
- Offline: badge nhỏ ở góc; widget dùng cache ghi "cập nhật lúc HH:MM".

### Concept voice-first: "mọi thứ sinh ra từ giọng nói"

Hub là một giao diện **điều khiển bằng giọng nói thật**: chạm chỉ là đường phụ. Ý tưởng xuyên suốt: một **Orb** sống neo ở góc phải dưới đại diện cho agent, và mọi thứ agent làm đều **sinh ra từ Orb** rồi đi tới chỗ của nó trên màn hình, để người dùng luôn thấy hành động nào đến từ câu nói nào.

**1. Orb — thực thể sống của agent**

| Trạng thái | Orb | Ghi chú |
|---|---|---|
| Idle / Ambient | Thở rất chậm (scale ±3%, chu kỳ ~4 s), nhỏ và mờ ở góc | Báo "đang sẵn sàng nghe" mà không gây chú ý |
| Wake | Bật sáng và lớn lên tại góc trong ~250 ms, sóng lan một vòng về phía màn | Phản hồi tức thì khi nghe `hey_jarvis`, trước cả khi server trả lời |
| Listening | Co giãn theo biên độ mic; vòng ngoài mờ lan theo tiếng nói | Người dùng thấy máy "đang nghe mình" |
| Thinking | Xoay chậm, các chấm nhỏ quay quanh | |
| Tool đang chạy | Bắn ra một "hạt" về phía nơi thẻ sẽ xuất hiện | Nối hình ảnh Orb → hành động |
| Speaking | Đập theo năng lượng audio TTS | |
| Lỗi / offline | Chuyển màu xám, rung nhẹ một lần | |

Orb vẽ bằng vài hình tròn chồng nhau với scale/opacity/rotation — rẻ với FemtoVG, không cần shader.

**2. Chữ phản hồi rung, chạy từ Orb sang trái (kinetic text)**

- Dòng chữ nằm ngang hàng với Orb, căn phải sát Orb.
- Mỗi từ mới **sinh ra từ Orb**: trượt vào từ bên phải, xoay/phóng nhẹ rồi rung tắt dần (~550 ms, overshoot), như được Orb "nói ra".
- Từ cũ bị đẩy dần sang trái và mờ ở mép trái (mask gradient); dòng chỉ giữ phần mới nhất, không xuống dòng.
- Nhịp hiện từ theo nhịp TTS; transcript của người dùng (`stt`) hiện cùng kiểu nhưng màu nhạt hơn để phân biệt ai đang nói.
- Phần quan trọng (thời gian, tên, con số) tô màu nhấn — như "luộc trứng **7 phút**".
- Đang nghe mà im lặng 10 giây thì phụ đề ẩn đi, hội thoại vẫn mở (Orb vẫn nghe). Khi hội thoại kết thúc (chạm Orb hoặc 5 phút im lặng), chữ mờ dần và Orb về Idle.
- Với FemtoVG: mỗi từ là một `Text` có transform/opacity riêng; chỉ animate vài từ mới nhất cùng lúc để giữ fps.

**3. Dải bước tool (tool trail)**

Ngay trên dòng chữ, cạnh Orb, là một dải chấm nhỏ: *nghe → hiểu → tên tool → xong*. Mỗi event `type:"tool"` thêm một chấm; `completed` chuyển xanh, `failed` chuyển đỏ. Cho người dùng thấy agent đang làm gì, nhất là với tool phía server mà hub không biết nội dung.

**4. Thẻ sinh ra từ Orb (spawn choreography)**

- Mọi kết quả (hẹn giờ, đếm ngược, sự kiện) trồi ra từ Orb ở góc phải dưới với hiệu ứng nảy (overshoot nhẹ ~600 ms) rồi bay vào **đúng vị trí của nó trên màn tính năng** đang hiện (xem "View theo giọng nói").
- Thẻ không thuộc tính năng nào (`hub_show_card` thông tin chung) nổi phía trên Voice dock và mờ đi khi hội thoại kết thúc.
- Thứ tự nhất quán: Orb → thẻ → chỗ của nó. Không có thứ gì "tự hiện ra" mà không đi qua Orb.

**5. Màn chính phản ứng theo giọng nói**

- Câu hỏi về một widget (vd. "mai có mưa không?") làm card đó phóng lớn tại chỗ trong lưới rồi trả về kích thước cũ — không cần màn con.
- Khi có hẹn giờ hoặc đếm ngược, màn chính tự dành một ô cho nó; hết hẹn giờ thì ô biến mất.
- Gợi ý câu nói: khi Idle, dòng chữ cạnh Orb xoay vòng vài câu mẫu ("Hey Jarvis, hẹn giờ 5 phút", "…còn bao lâu đến Tết?") — dạy người dùng nói thay vì chạm.

**6. Âm thanh giao diện**

Tiếng "ding" ngắn khi wake, tiếng nhẹ khi tool xong, chuông khi hẹn giờ hết. Phát qua cùng đường loa với TTS.

### Agent điều khiển hub và hiệu ứng tool

Hub là **thiết bị do AI agent điều khiển**: agent trên vela-server vừa trả lời bằng giọng, vừa điều khiển màn hình hub qua tool. Mỗi lần agent gọi tool, hub hiện hiệu ứng tương ứng.

Có hai loại tool, hub biết được lượng thông tin khác nhau:

| Loại | Chạy ở | Hub nhận được | Hiệu ứng làm được |
|---|---|---|---|
| **Device MCP** (tool của hub) | Trên hub; server gọi `tools/call` | Toàn bộ arguments | Hiệu ứng đầy đủ, có nội dung (tiêu đề, giờ…) |
| Tool phía server (Builtin, External MCP như Home Assistant, Google Calendar) | Trên server | Chỉ `type:"tool"` với `name` và `state` (`started`/`completed`/`failed`), không có arguments/results | Hiệu ứng chung theo tên tool (icon, nhóm) |

ClientHello của hub khai báo `features.mcp=true` (bật Device MCP), `features.tool_status=true` (nhận event tool phía server) và `features.direct_tts=true` (dùng message `direct_speech` để đọc Reminder).

**Tool Device MCP (đã chốt)** — mỗi tool một việc:

| Tool | Việc | Hiệu ứng |
|---|---|---|
| `hub_show_card` | Hiện thẻ nội dung (sự kiện, Reminder, thông tin) do agent soạn | Thẻ sinh ra từ Orb; thẻ thuộc tính năng thì mở màn đó và bay vào chỗ của nó |
| `hub_weather_get` | Trả dự báo (hiện tại, theo giờ, 7 ngày, AQI); tham số `focus` (ngày/giờ), `location` (tên thành phố, tuỳ chọn) | Mở màn Thời tiết của vị trí đó, làm nổi bật ngày/giờ được hỏi |
| `hub_calendar_get` | Trả sự kiện trong khoảng thời gian (từ cache Google Calendar API) | Mở màn Lịch, làm nổi bật ngày/sự kiện được hỏi |
| `hub_calendar_create` | Tạo sự kiện trên Google Calendar: tiêu đề, bắt đầu, kết thúc hoặc cả ngày, địa điểm (tuỳ chọn) | Mở màn Lịch; thẻ sự kiện sinh ra từ Orb, bay vào đúng ngày |
| `hub_timer_list` | Trả các hẹn giờ đang chạy | Mở màn Hẹn giờ |
| `hub_countdown_list` | Trả các Ngày đặc biệt | Mở màn Ngày đặc biệt, làm nổi bật mốc được hỏi |
| `hub_open_screen` | Mở màn tính năng (thời tiết, lịch, nhà, hẹn giờ, ngày đặc biệt, Reminder, ảnh, setting) hoặc về màn trung tâm | Chuyển màn như khi vuốt |
| `hub_set_brightness` / `hub_set_volume` | Chỉnh độ sáng / âm lượng | Thanh trượt hiện và chạy tới giá trị mới |
| `hub_timer_start` | Hẹn giờ theo khoảng thời gian (vd. 7 phút), có nhãn | Mở màn Hẹn giờ; vòng đếm ngược mới sinh ra từ Orb vào danh sách; hết giờ thì chuông + Orb nhấp nháy |
| `hub_timer_cancel` | Huỷ một hẹn giờ theo nhãn | Mở màn Hẹn giờ; vòng tròn co lại rồi biến mất |
| `hub_timer_adjust` | Chỉnh một hẹn giờ theo nhãn: thêm/bớt thời gian (`delta_seconds`), tạm dừng, tiếp tục | Mở màn Hẹn giờ; vòng nở ra/co lại, hoặc mờ đi/sáng lại |
| `hub_countdown_create` | Lưu một Ngày đặc biệt (vd. Tết, sinh nhật), có nhãn; `repeat`: `none` / `yearly` / `yearly_lunar` | Mở màn Ngày đặc biệt; ô mới sinh ra từ Orb |
| `hub_countdown_remove` | Xoá một Ngày đặc biệt theo nhãn | Mở màn Ngày đặc biệt; ô mờ dần |
| `hub_reminder_create` | Tạo Reminder cục bộ: nội dung, thời điểm, `repeat` (`none` / `daily` / `weekly`) | Mở màn Reminder; mục mới sinh ra từ Orb vào dòng thời gian |
| `hub_reminder_list` | Trả các Reminder sắp tới; tham số `focus` (khoảng thời gian) | Mở màn Reminder, làm nổi bật các mục được hỏi |
| `hub_reminder_cancel` | Huỷ một Reminder | Mục mờ dần |
| `hub_reminder_snooze` | Hoãn Reminder đang nhắc thêm N phút | Trạng thái nhắc thu lại, mục dời giờ |
| `hub_show_image` | Hiện 1–6 ảnh từ URL, kèm chú thích từng ảnh | Ảnh toàn màn, zoom chậm (Ken Burns), Voice dock vẫn ở góc phải dưới; tự đóng khi hội thoại kết thúc hoặc chạm |
| `hub_image_list` | Trả các ảnh trong cache (id, chú thích, thời điểm, đã ghim) | Mở màn Ảnh |
| `hub_image_control` | Với ảnh đang xem hoặc theo id: mở, tiếp, trước, ghim, bỏ ghim | Chuyển ảnh / ghim nhảy lên |

#### View theo giọng nói

Quy tắc chung cho mọi tính năng: **giọng nói điều khiển view.** Khi agent gọi một tool gắn với một tính năng — đọc (`*_get`, `*_list`) hay ghi (`*_start`, `*_create`…) — hub **chuyển tới màn của tính năng đó** và làm nổi bật phần liên quan, trong khi câu trả lời chạy cạnh Orb.

```
"Hey Jarvis, mai có mưa không?"
  → LLM gọi hub_weather_get(focus = ngày mai)
  → hub chuyển sang màn Thời tiết (~400 ms, như vuốt) và trả dữ liệu cho agent
  → hàng "Mai" sáng lên, đường 24 giờ chuyển sang ngày mai
  → TTS: "Mai có mưa nhẹ buổi sáng…" chạy từ Orb sang trái
  → hội thoại kết thúc → ở lại màn đó → ~60 giây không tương tác → về màn trung tâm
```

| Tính năng | Tool đọc | Tool ghi | Nổi bật khi mở |
|---|---|---|---|
| Thời tiết | `hub_weather_get` | — | Ngày/giờ được hỏi |
| Lịch | `hub_calendar_get` | `hub_calendar_create` | Ngày/sự kiện được hỏi hoặc vừa tạo |
| Hẹn giờ | `hub_timer_list` | `hub_timer_start`, `hub_timer_cancel`, `hub_timer_adjust` | Hẹn giờ vừa tạo/huỷ |
| Ngày đặc biệt | `hub_countdown_list` | `hub_countdown_create`, `hub_countdown_remove` | Mốc được hỏi hoặc vừa tạo |
| Ảnh | `hub_image_list` | `hub_show_image`, `hub_image_control` | Ảnh toàn màn hoặc ảnh được nhắc tới |
| Setting | — | `hub_set_brightness`, `hub_set_volume` | Không đổi màn; chỉ hiện thanh trượt |
| Reminder | `hub_reminder_list` | `hub_reminder_create`, `hub_reminder_cancel`, `hub_reminder_snooze` | Mục được hỏi hoặc vừa tạo |
| Nhà | (để sau, cùng điều khiển nhà bằng giọng) | — | — |

Quy tắc chi tiết:

- **Tool đọc trả đúng dữ liệu đang hiển thị**, để số agent nói ra khớp với màn hình. Agent không tự lấy thời tiết/lịch từ nguồn khác cho các câu hỏi này (chỉ dẫn trong prompt/Template của agent).
- **Tham số `focus`** của tool đọc (ngày, giờ, nhãn) quyết định phần được làm nổi bật.
- **Tool phía server** (hub chỉ nhận `name` + `state`): hub giữ một bảng ánh xạ tên tool → màn (vd. tool lịch phía server → màn Lịch). Khi event `started` tới, hub chuyển màn; dữ liệu hiển thị lấy từ cache của hub.
- **Nhiều tool trong một lượt:** chuyển màn theo thứ tự gọi; màn của tool cuối cùng ở lại.
- **Người dùng đang thao tác** (vừa chạm trong ~3 giây): vẫn chuyển màn vì người dùng vừa hỏi bằng giọng — giọng nói ưu tiên hơn chạm.
- **Ambient:** tool đánh thức màn hình (độ sáng bình thường) trước khi chuyển màn.

Ghi chú:

- Hẹn giờ và đếm ngược chạy và lưu **cục bộ trên hub**, nên vẫn hoạt động khi mất mạng và còn sau khi khởi động lại.
- Huỷ/xoá đi kèm vì người dùng sẽ nói "huỷ hẹn giờ" — thiếu nó thì agent không làm được.
- `hub_show_image` nhận **URL** (không nhận ảnh base64 qua WebSocket). Hub tự tải với giới hạn: chỉ `http(s)`, kích thước tối đa (vd. 10 MB), timeout, chỉ JPEG/PNG/WebP; thu nhỏ về độ phân giải màn hình trước khi hiển thị để tiết kiệm RAM. Lỗi tải → thẻ báo lỗi thay vì ảnh.
- Ảnh đến từ đâu (tìm ảnh trên web, ảnh từ Home Assistant camera…) là việc của tool phía server; hub chỉ hiển thị.

**Ví dụ "tạo sự kiện":** agent gọi `hub_calendar_create` (tool của hub, có đủ tiêu đề và giờ).

1. Hub nhận `tools/call` → chuyển sang màn Lịch; thẻ sự kiện (đã có nội dung) trồi ra từ Orb, ở trạng thái "đang lưu".
2. Hub ghi lên Google Calendar API → thành công: dấu check, thẻ bay vào đúng ngày; hub trả kết quả cho agent.
3. Thất bại (mất mạng, token hết hạn): thẻ rung nhẹ, viền đỏ; hub trả lỗi để agent nói lại cho người dùng.

Không cần `hub_show_card` cho trường hợp này vì hub đã có đủ dữ liệu từ arguments của tool.

### Setting

Setting chia hai nơi, cùng ghi xuống một file config trên thiết bị:

**Trên màn hình (chỉ chọn, không nhập chữ)** — không cần bàn phím ảo. Bố cục: danh mục dọc bên trái, bảng điều khiển ở giữa, ô QR **Setup web** cố định bên phải (IP:cổng + mã QR + "Server, lịch, nhà, vị trí").

| Danh mục | Mục |
|---|---|
| Hiển thị | Độ sáng (thanh trượt); tự chỉnh sáng theo giờ (tắt / theo giờ); độ phân giải (720p / 1080p / 1440p — chỉ liệt kê mode màn hình hỗ trợ); cỡ chữ (nhỏ / vừa / lớn → `scale-factor`); vào Ambient sau (1 / 5 / 10 phút / không); Ambient ban đêm (bật/tắt, khung giờ chọn sẵn) |
| Âm thanh | Âm lượng; âm thanh giao diện (bật/tắt); chuông hẹn giờ và Reminder (chọn 1 trong vài âm có sẵn, chạm để nghe thử) |
| Giọng nói | Wake word (bật/tắt); **độ nhạy wake word** (thấp / vừa / cao → threshold); **tắt mic** (Orb chuyển xám có gạch, wake word ngừng hẳn — chỉ bật lại bằng chạm); đọc Reminder bằng giọng server (bật/tắt) |
| Nhà | Chọn entity demo từ danh sách đã lấy từ Home Assistant (bật/tắt từng entity, tối đa 4 trên màn trung tâm) |
| Kết nối | Trạng thái vela-server (đã kết nối / mất kết nối, độ trễ); mã Enrollment khi chưa đăng ký; Wi-Fi (tên mạng, cường độ, IP); Home Assistant, Google Calendar, Open-Meteo (lần lấy dữ liệu gần nhất, lỗi gần nhất) |
| Dữ liệu | Xoá toàn bộ ảnh; xoá cache thời tiết/lịch; xoá hẹn giờ, đếm ngược, Reminder (mỗi mục một nút, có hỏi lại) |
| Hệ thống | Phiên bản app; múi giờ (danh sách chọn, mặc định Asia/Ho_Chi_Minh; giờ lấy qua NTP); khởi động lại app; khởi động lại máy; tắt máy |

Quy tắc:

- **Áp dụng ngay**, không có nút "Lưu".
- **Đổi độ phân giải** → màn đổi mode rồi hiện "Giữ độ phân giải này? 10" đếm ngược; không chạm "Giữ" thì tự quay về mode cũ (phòng màn hình không hiển thị được).
- Thao tác xoá và tắt/khởi động lại → hỏi lại một lần bằng thẻ xác nhận.
- **Cử chỉ:** kéo thanh trượt không bị hiểu là vuốt điều hướng — cử chỉ bắt đầu trên một điều khiển ngang thuộc về điều khiển đó.
- Bảng điều khiển cuộn dọc nếu dài; danh mục luôn hiện.
- **Giọng nói:** độ sáng, âm lượng qua `hub_set_brightness` / `hub_set_volume` (không đổi màn, chỉ hiện thanh trượt); các mục khác chỉnh bằng chạm. Không có tool tắt mic bằng giọng nói (tắt rồi thì không bật lại bằng giọng được).

**Lần đầu khởi động (chưa có Wi-Fi) — đã chốt:** Setup web cần mạng LAN, mà nhập mật khẩu Wi-Fi lại cần gõ chữ. Khi chưa có Wi-Fi, hub tự phát một điểm truy cập (vd. "AI-Hub-Setup") và hiện **hai mã QR** — QR 1 để điện thoại vào mạng đó, QR 2 mở trang Setup web — nhập Wi-Fi nhà xong thì hub chuyển sang mạng nhà và hiện IP mới. Có dây mạng (Ethernet) thì bỏ qua bước này.

**Setup web qua LAN (setting chi tiết, có nhập chữ)** — app tự phục vụ một trang web nhỏ, mở từ điện thoại hoặc máy tính cùng mạng:

- Wi-Fi (tên mạng, mật khẩu) — cũng dùng ở lần đầu khởi động.
- Địa chỉ vela-server.
- Vị trí thời tiết (toạ độ / tìm theo tên).
- Kết nối Google Calendar (OAuth).
- Địa chỉ và token Home Assistant.

Setup web **luôn mở cho LAN, không có xác thực** (quyết định có chủ ý: thiết bị dùng trong mạng nhà). Hệ quả: bất kỳ máy nào trong LAN đều xem và sửa được setting, kể cả token.

## 7. Hệ thống và kiosk

- Debian 13 trixie, không cài desktop.
- App chạy bằng systemd service, `Restart=always`, user riêng trong nhóm `video input render audio`.
- Tắt `getty@tty1`, dịch vụ không dùng; ẩn log và con trỏ trên console; splash khi boot.
- Sau này có thể chuyển Buildroot/Yocto để boot nhanh hơn.

## 8. Cấu hình và bí mật

Lưu trên thiết bị, quyền đọc chỉ cho user của app:

- Token Voice/OTA của vela-server.
- Refresh token Google Calendar.
- Long-lived token Home Assistant.
- Mật khẩu Wi-Fi.

## 9. Rủi ro

| Rủi ro | Giảm thiểu |
|---|---|
| Không có AEC phần cứng → wake word tự kích hoạt bởi loa, barge-in sai | Chưa có; cần chọn cách khử vọng cho mic/loa tích hợp (ADR 0005) |
| Hiệu ứng giật trên GPU tích hợp | Tránh blur, đo fps sớm |
| Độ trễ khi đánh thức | Giữ WS ở Ready, ring buffer sau wake word |
| `hey_jarvis` train bằng giọng tiếng Anh → kém nhạy với giọng Việt | Chỉnh threshold khi thử thực tế; đo tỉ lệ bỏ sót và kích hoạt nhầm |
| `hub_show_image` tải URL do LLM đưa → ảnh nặng làm đầy RAM, hoặc hub bị dùng để gọi địa chỉ trong LAN | Giới hạn kích thước, timeout, loại file; thu nhỏ trước khi hiển thị; cân nhắc chặn địa chỉ LAN/localhost |
| Lộ token HA / refresh token Google | Quyền file chặt; không log giá trị. Setup web mở cho LAN nên chấp nhận rủi ro trong mạng nhà |

## 10. Câu hỏi mở

- vela-server: duyệt ADR 0161 (message `direct_speech`) rồi triển khai issue `.scratch/client-direct-speech/issues/01`.
## 11. Tham chiếu trong vela-server

- `AI_Agent_Voice_rust/docs/flows/01-websocket.md` — admission, handshake, lifecycle Voice Session.
- `AI_Agent_Voice_rust/docs/flows/02-vad.md` — auto/realtime/manual mode, barge-in, `features.aec`.
- `AI_Agent_Voice_rust/docs/api/studio-assistant.md` — message, Opus 60 ms, trạng thái turn.
- `AI_Agent_Voice_rust/apps/admin-web/src/lib/assistant/` — client Voice Protocol bằng TypeScript.
- `AI_Agent_Voice_rust/firmware/esp32` — client vật lý chuẩn.
