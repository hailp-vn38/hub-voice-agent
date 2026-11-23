# 19 — Lần đầu khởi động: hai QR

Status: resolved
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Setting" (Lần đầu khởi động), [ADR 0010](../../../docs/adr/0010-first-boot-access-point-two-qr.md)

## Việc

- `screens/first-boot.slint`, hiện khi chưa có Wi-Fi và Ethernet.
- Bước 1: QR 1 vào mạng điểm truy cập (vd. "AI-Hub-Setup") kèm tên mạng in rõ; QR 2 mở Setup web kèm địa chỉ in rõ; hướng dẫn ngắn hai bước.
- Bước 2: đang chuyển sang Wi-Fi nhà (trạng thái chờ, tên mạng).
- Bước 3: đã vào mạng nhà, hiện IP mới + QR Setup web mới, nút "Xong" → `done()`.
- Lỗi kết nối Wi-Fi nhà → quay về bước 1 kèm dòng báo lỗi.
- Không có Voice dock tương tác ở màn này nếu voice chưa cấu hình (Orb xám).
- `contract/first-boot.slint`: `export enum BootStep { access-point, joining, joined }`; `global FirstBootData { in property <BootStep> step; in property <string> ap-name; in property <image> ap-qr; in property <string> setup-url; in property <image> setup-qr; in property <string> home-network; in property <string> error; callback done(); }`.

## Demo

Kịch bản: ba bước nối tiếp, "Sai mật khẩu Wi-Fi". QR demo sinh bằng crate `qrcode` từ chuỗi giả.

## Nghiệm thu

- [ ] Hai QR quét được bằng điện thoại từ khoảng 50 cm (ready-for-human trên Pi).
- [ ] Mỗi bước có chữ hướng dẫn đủ để làm không cần tài liệu.

## Map với hub-core

Map: chưa có ticket hub-core (ADR 0010, port `System`)

| Hợp đồng | UiState / UserInput |
|---|---|
| `FirstBootData.step` | |
| `.ap-name` / `.ap-qr` | |
| `.setup-url` / `.setup-qr` | |
| `.home-network` / `.error` | |
| `done()` | |

## Answer

Đã thêm `contract/first-boot.slint`, `screens/first-boot.slint` (hiện qua Shell khi `Nav.screen == first-boot`), fixture demo `fixtures/first_boot.rs` và hai kịch bản ("Lần đầu: 3 bước", "Lần đầu: sai mật khẩu"). QR demo sinh bằng `qrcode` (optional, bật bởi feature `demo`) trong `demo/qr.rs`, có unit test. Nghiệm thu quét QR từ 50 cm còn chờ thử trên Pi.
