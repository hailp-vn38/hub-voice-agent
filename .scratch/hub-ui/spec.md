# Spec: hub UI (`apps/hub`, Slint)

Status: needs-triage

## Mục tiêu

Dựng toàn bộ giao diện Slint của hub **đúng thiết kế đã chốt** ở [concept.md §6](../../docs/concept.md) ("UI và hiển thị"), song song với [spec hub-core](../hub-core/spec.md). UI làm trước với **dữ liệu demo**. Khi một ticket hub-core xong, nối `UiState` thật vào cùng hợp đồng đó (bước **map**) mà không phải sửa màn hình.

Xong spec này thì:

- Mọi màn, overlay và hiệu ứng trong concept.md §6 chạy được trên Mac ở chế độ demo, kèm kịch bản bấm thử cho từng hiệu ứng.
- Mỗi màn đọc dữ liệu từ một **hợp đồng Slint** cố định (struct + global + callback). Map với hub-core chỉ chạm `src/ui/bridge/`.
- Hiệu ứng nặng đã đo fps trên Pi 4 thật.

## Seam: hợp đồng Slint

UI và hub-core gặp nhau ở **hợp đồng Slint** của từng tính năng, không gặp ở `UiState`. Hai bên làm song song được là nhờ seam này. Hợp đồng có hai adapter: **demo** (làm ngay) và **bridge** (map khi hub-core xong).

```
            ┌── src/ui/demo/   (fixture + kịch bản)    ← làm trong spec này
contract ◄──┤
            └── src/ui/bridge/ (UiState → contract)    ← bước map, sau ticket hub-core
```

- `apps/hub/ui/contract/<feature>.slint`: struct, enum và một `export global <Feature>Data` gồm `in property` (dữ liệu) và `callback` (thao tác người dùng). Mỗi tính năng một file.
- Màn và component **chỉ đọc global của hợp đồng**, không nhận dữ liệu qua chuỗi property từ `App`.
- Rust ghi vào global qua `app.global::<TimersData>().set_…`. Demo và bridge ghi cùng một global.
- Callback trong hợp đồng là ý định của người dùng (`cancel(key)`, `navigate(screen)`), không phải lệnh nghiệp vụ. Bridge đổi callback thành `UserInput`; demo tự xử lý để kịch bản chạy được.

### Quy tắc hợp đồng

1. **Dữ liệu đã sẵn để hiển thị.** Chuỗi đã định dạng tiếng Việt ("xong lúc 14:32", "Thứ Sáu", "còn 25 phút"), số chuẩn hoá (`progress` 0..1), enum trạng thái. Slint không tính luật nghiệp vụ: không so ngày, không đếm mốc, không chọn "mốc gần nhất". Slint chỉ tính layout và animation.
2. **Khoá ổn định.** Mỗi mục trong danh sách có `key: string` không đổi suốt đời mục. Animation thêm, xoá, dời chỗ và focus đều dựa trên `key`.
3. **Hiệu ứng một lần = serial tăng dần** ([ADR 0002](../../docs/adr/0002-single-process-actor-ui-reads-snapshot.md)): `spawn-serial`, `focus-serial`, `wake-serial`… Slint chạy animation trong `changed <serial> => { … }`. Cùng giá trị gửi lại thì không chạy lại.
4. **Thời gian chạy giữa hai tick do Slint nội suy.** Dữ liệu cập nhật 1 Hz. Vòng hẹn giờ dùng `animate` tuyến tính 1 s, không tự đếm giờ.
5. **Tần số cao đi đường riêng.** Biên độ mic và năng lượng TTS nằm trong global `VoiceLevels` (~30 Hz), tách khỏi dữ liệu 1 Hz.
6. **Màn hiện tại do hub-core quyết** (`view`). UI gửi `Nav.navigate(screen)` khi người dùng vuốt hoặc chạm, và hiển thị ngay màn đích trong lúc chờ; `Nav.screen` là nguồn sự thật. Tự quay về sau ~60 s, Ambient và chuyển màn theo tool đều đến qua `Nav.screen`.
7. **Ảnh** (thumbnail, ảnh xem, QR) là `image` trong hợp đồng; Rust giải mã hoặc sinh ảnh, Slint không tải URL.
8. **Chữ cố định** (tiêu đề, gợi ý câu nói theo màn, nhãn nút) nằm trong `.slint`. Chữ phụ thuộc dữ liệu thì có trong hợp đồng.

## Cấu trúc

```
apps/hub/
├── ui/
│   ├── app.slint            # cửa sổ gốc: tầng màn, Voice dock, lớp phủ
│   ├── theme.slint          # token màu, cỡ chữ, khoảng cách theo scale-factor
│   ├── contract/            # một file mỗi tính năng: struct + global <Feature>Data
│   ├── components/          # orb, response-text, tool-trail, card, slider, page-dots, spawn, qr…
│   ├── screens/             # center, menu, weather, calendar, home, timers, countdowns,
│   │                        # reminders, photos, settings, ambient, first-boot
│   ├── overlays/            # alert, voice-slider, confirm
│   └── demo/panel.slint     # cửa sổ thứ hai: nút kịch bản demo
├── assets/{fonts,icons,sounds}/
└── src/ui/
    ├── mod.rs               # run_ui(); chọn demo hoặc bridge
    ├── bridge/              # mod.rs + <feature>.rs: UiState → global (bước map)
    └── demo/                # feature `demo`: mod.rs, fixtures/<feature>.rs, scenarios.rs
```

- **Feature `demo`** của `apps/hub` (cộng thêm vào `desktop`, không dùng cho `pi`): `scripts/run.sh run --features demo` mở app với fixture và cửa sổ **Demo panel** có nút cho từng kịch bản (wake, nói, tool, hẹn giờ mới, hết giờ, offline…). Không có hub-core và voice-client nào chạy trong chế độ này.
- Tuân theo AGENTS.md: một màn một file `.slint`, file < 300 dòng, tách component dùng chung vào `components/`.

## Quyết định cho spec này

- **Slint 1.19** đang dùng: `SwipeGestureHandler` cho vuốt theo ngón tay, `changed` cho serial, global cho hợp đồng.
- **Font** Be Vietnam Pro (OFL), hai độ đậm, nhúng vào binary. **Icon** Tabler outline (MIT), xuất SVG vào `assets/icons/`.
- **Đơn vị logic 1920×1080**, cỡ theo `Theme` nhân hệ số cỡ chữ (nhỏ / vừa / lớn). Vùng chạm ≥ 88 px logic.
- **Nền tối**, một màu nhấn cho voice, một màu ấm cho thiết bị đang bật và cảnh báo ([concept.md "Phong cách"](../../docs/concept.md)).
- **Hiệu ứng chỉ dùng transform, opacity, gradient**; không blur. Hiệu ứng nặng (nền thời tiết, pháo giấy, Ken Burns, chữ phản hồi) có mục đo fps trên Pi trong nghiệm thu.
- **QR** sinh ở Rust (crate `qrcode`) thành `image`.
- **Không tự động test giao diện.** Nghiệm thu bằng kịch bản demo trên Mac + đo trên Pi. Rust trong `demo/` và `bridge/` có unit test cho hàm chuyển đổi khi có logic.

## Lát dọc (ticket)

| # | Ticket | Phụ thuộc |
|---|---|---|
| 01 | [Hợp đồng, chế độ demo, Demo panel](issues/01-contract-demo.md) | — |
| 02 | [Theme, font, icon](issues/02-theme-assets.md) | 01 |
| 03 | [Khung app: tầng màn, vuốt, chấm trang](issues/03-shell-navigation.md) | 02 |
| 04 | [Voice dock: Orb](issues/04-orb.md) | 03 |
| 05 | [Chữ phản hồi, dải bước tool, gợi ý câu nói](issues/05-response-text-tool-trail.md) | 04 |
| 06 | [Thẻ sinh ra từ Orb, thẻ nổi](issues/06-spawn-cards.md) | 04 |
| 07 | [Màn trung tâm](issues/07-center.md) | 03, 06 |
| 08 | [Menu tính năng](issues/08-menu.md) | 03 |
| 09 | [Màn Thời tiết](issues/09-weather.md) | 06 |
| 10 | [Màn Lịch](issues/10-calendar.md) | 06 |
| 11 | [Màn Hẹn giờ](issues/11-timers.md) | 06 |
| 12 | [Màn Đếm ngược](issues/12-countdowns.md) | 06 |
| 13 | [Màn Reminder](issues/13-reminders.md) | 06 |
| 14 | [Lớp báo và thanh trượt giọng nói](issues/14-alert-overlays.md) | 04 |
| 15 | [Màn Ảnh và xem ảnh](issues/15-photos.md) | 06 |
| 16 | [Màn Nhà](issues/16-home.md) | 03 |
| 17 | [Setting](issues/17-settings.md) | 03 |
| 18 | [Ambient](issues/18-ambient.md) | 03 |
| 19 | [Lần đầu khởi động: hai QR](issues/19-first-boot.md) | 03 |

Ticket 07–19 độc lập với nhau sau khi phụ thuộc xong, làm song song được. Mỗi ticket chỉ sửa file của nó trong `ui/contract/`, `ui/screens/`, `ui/overlays/` và `src/ui/demo/fixtures/`. File chung (`app.slint`, `demo/scenarios.rs`) chỉ thêm một dòng đăng ký.

## Map với hub-core

Mỗi ticket có mục **Map với hub-core**: bảng từng field và callback của hợp đồng, cột `UiState` / `UserInput` để trống. Khi ticket hub-core tương ứng xong:

1. Điền bảng ở ticket UI, ghi `Map: done` kèm commit.
2. Viết `src/ui/bridge/<feature>.rs`: `apply(&app, &state.<feature>)` và nối callback → `UserInput`. Chuyển đổi chỉ là định dạng, không có nhánh nghiệp vụ; cần nhánh thì đưa vào hub-core.
3. Chạy app thật (không có `demo`) và lặp lại kịch bản của ticket bằng giọng hoặc chạm.

| Hợp đồng | Ticket UI | Ticket hub-core | Map |
|---|---|---|---|
| `Nav` | 03 | 05 (view), 14 (present, focus serial); `hub_open_screen` **chưa có ticket** | chưa |
| `Voice`, `VoiceLevels` | 04, 05 | 03 (conversation), 15; levels từ audio ở `apps/hub` | chưa |
| `ToolTrail` | 05 | 03, 05 (`ServerTool`), 15 | chưa |
| `Spawn`, `FloatingCards` | 06 | 04 trở đi; `hub_show_card` **chưa có ticket** | chưa |
| `Center` | 07 | 05, 06, 09, 10, 11, 21 | chưa |
| `Menu` | 08 | 05, 21 | chưa |
| `WeatherData` | 09 | 09, 18 | chưa |
| `CalendarData` | 10 | 10, 08 (Reminder trên dòng thời gian), 19 | chưa |
| `TimersData` | 11 | 04, 16 | chưa |
| `CountdownsData` | 12 | 07, 17 | chưa |
| `RemindersData` | 13 | 08, 17 | chưa |
| `AlertData`, `VoiceSlider` | 14 | 06, 08, 16; `hub_set_brightness/volume` **chưa có ticket** | chưa |
| `PhotosData` | 15 | 12, 20 | chưa |
| `HomeData` | 16 | 11, 20 | chưa |
| `SettingsData` | 17 | 02 (Kết nối voice), 13; các mục khác **chưa có ticket** | chưa |
| `StatusBar` (Wi-Fi, offline, giờ) | 03 | 03, 06, 14; Wi-Fi qua `System` **chưa có ticket** | chưa |
| `AmbientData` | 18 | 05, 14 | chưa |
| `FirstBootData` | 19 | ADR 0010, **chưa có ticket** | chưa |

Ticket hub-core 14–21 thêm lớp `present` (hàm thuần `UiState` → `HubView`) để bridge chỉ copy field, không định dạng hay suy ra. Các dòng **chưa có ticket** là chỗ hub-core còn thiếu, liệt kê chi tiết (G1–G11) ở [hub-core-gaps.md](hub-core-gaps.md). UI vẫn làm được với demo; cần thêm ticket hub-core trước khi map.

## Ngoài phạm vi

- Logic hub-core, adapter audio thật, adapter `System` cho Pi.
- Đổi độ phân giải lúc chạy: chỉ làm giao diện chọn và thẻ "Giữ độ phân giải này? 10"; áp mode KMS là câu hỏi mở ở [codebase.md §7](../../docs/codebase.md).
- Theme sáng, điều khiển nhà bằng giọng nói.
- Setup web (HTML), thuộc hub-core #13.

## Quy ước

- Mỗi ticket ghi rõ mục concept.md mà nó hiện thực; khác concept thì sửa concept.md hoặc ghi lý do vào `## Comments` của ticket, không làm lệch âm thầm.
- Thêm struct hay field vào hợp đồng thì cập nhật bảng Map của ticket đó.
