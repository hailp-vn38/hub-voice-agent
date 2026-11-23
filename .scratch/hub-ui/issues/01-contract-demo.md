# 01 — Hợp đồng, chế độ demo, Demo panel

Status: ready-for-human
Blocked by: —
Spec: [../spec.md](../spec.md)

## Việc

Dựng seam "hợp đồng Slint" và adapter demo để mọi ticket sau làm được mà không cần hub-core.

- `ui/contract/nav.slint`: `export enum Screen { center, menu, weather, calendar, home, timers, countdowns, reminders, photos, settings, ambient, first-boot }`; `export global Nav { in property <Screen> screen; callback navigate(Screen); }`.
- Mỗi file hợp đồng mở đầu bằng một comment trỏ về ticket UI của nó; quy tắc hợp đồng nằm ở spec.
- Feature `demo` trong `apps/hub/Cargo.toml`. `src/ui/mod.rs` chọn: có `demo` → `demo::start(&app)`, không có → bridge như hiện nay. Đang có voice settings chạy thật: giữ nguyên đường thật, không phá.
- `src/ui/demo/`:
  - `mod.rs`: `start(app)` ghi fixture vào mọi global, mở cửa sổ Demo panel, tick 1 Hz cho dữ liệu chạy theo giờ.
  - `fixtures/<feature>.rs`: mỗi tính năng một file, ticket sau thêm.
  - `scenarios.rs`: bảng `(tên nút, fn(&App))`; ticket sau đăng ký kịch bản của mình bằng một dòng.
  - Demo tự xử lý callback (vd. `Nav.navigate(s)` → `set_screen(s)`) để kịch bản tự nhiên.
- `ui/demo/panel.slint`: cửa sổ thứ hai, cột nút sinh từ danh sách tên kịch bản; callback `run(index)`.
- `scripts/run.sh run --features demo` chạy được; ghi một dòng vào `docs/development.md` §3.

## Nghiệm thu

- [ ] `cargo build -p hub` và `cargo build -p hub --features demo` đều qua; bản Pi (`--no-default-features --features pi`) không biên dịch `src/ui/demo/`.
- [ ] Demo mở cửa sổ app + Demo panel; nút "Mở menu" / "Về trung tâm" đổi `Nav.screen` (màn tạm là chữ tên màn).
- [ ] Settings → Kết nối voice ở chế độ không demo vẫn chạy như trước.
- [ ] Thêm một kịch bản mới chỉ cần sửa `scenarios.rs` và file fixture của tính năng.

## Map với hub-core

Map: chờ hub-core #05

| Hợp đồng | UiState / UserInput |
|---|---|
| `Nav.screen` | |
| `Nav.navigate(Screen)` | |

## Answer

Đã dựng `ui/contract/nav.slint`, feature `demo`, `src/ui/demo/` (mod, fixtures/mod.rs, scenarios.rs, clock), `ui/demo/panel.slint`; App có `demo-mode` hiện tên màn. Đường thật không đổi. Ghi chú: `DemoPanel` được export từ app.slint nên Slint vẫn biên dịch panel trên Pi, chỉ Rust demo bị loại; đồng hồ demo theo UTC.

## Comments

- 2026-11-23 (review hub-ui): Tổ chức lại `src/ui/demo/`: `mod.rs` chỉ khai báo; `start.rs` khởi động demo; mọi tính năng nằm trong `fixtures/<feature>` (`fixtures/wiring.rs` nạp và tick). `ui/demo/` chứa panel, sample, spawn-demo. `App` bỏ `demo-mode` và các property voice cũ; app thật cũng chạy Shell. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
