# 14 — Lớp báo và thanh trượt giọng nói

Status: ready-for-human
Blocked by: 04
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn Hẹn giờ" (Hết giờ), "Màn Reminder" (Lúc Reminder đến hạn), "Agent điều khiển hub" (`hub_set_brightness` / `hub_set_volume`), "Setting" (thẻ xác nhận)

## Việc

- `overlays/alert.slint`: trạng thái báo toàn màn, phủ mọi màn kể cả Ambient.
  - Hết giờ: nền ấm, icon chuông với vòng lan theo nhịp, "{nhãn} · hết giờ".
  - Reminder: cùng kiểu, khác màu; icon chuông, giờ, nội dung chạy ra từ Orb.
  - **Chạm bất kỳ đâu** → `dismiss()`. Voice dock vẫn hiện trên lớp báo.
  - Khi có báo, Orb nhấp nháy theo nhịp chuông (concept, `hub_timer_start`).
  - Một lúc chỉ một báo; hàng đợi, tự tắt, đợi hết lượt nói do hub-core quyết.
  - Báo đóng: Reminder thu về "Đã nhắc gần đây" (thu nhỏ về phía màn Reminder/góc), hẹn giờ mờ đi.
- `overlays/voice-slider.slint`: thanh trượt độ sáng hoặc âm lượng hiện ở màn hiện tại, chạy tới giá trị mới, tự ẩn sau vài giây. Không đổi màn.
- `overlays/confirm.slint`: thẻ xác nhận dùng chung (tiêu đề, nội dung, "Huỷ" / nút hành động) cho xoá và tắt máy (ticket 17 dùng).
- `contract/alert.slint`:
  - `export enum AlertKind { none, timer, reminder }`.
  - `global AlertData { in property <AlertKind> kind; in property <int> serial; in property <string> title; in property <string> time; in property <string> body; callback dismiss(); }`.
  - `export enum SliderKind { brightness, volume }`; `global VoiceSlider { in property <SliderKind> kind; in property <float> value; in property <int> serial; }`.

## Demo

Kịch bản: "Hết giờ trứng", "Reminder uống thuốc", "Hai báo liên tiếp" (demo đổi `serial`), "Báo khi đang Ambient", "Độ sáng 30%", "Âm lượng 80%".

## Nghiệm thu

- [ ] Báo phủ mọi màn kể cả Ambient; chạm bất kỳ đâu đóng báo.
- [ ] Hết giờ và Reminder phân biệt rõ bằng màu.
- [ ] Vòng lan theo nhịp không làm tụt fps trên Pi (ready-for-human).
- [ ] Thanh trượt chạy tới giá trị mới rồi tự ẩn; màn bên dưới không đổi.

## Map với hub-core

Map: chờ hub-core #06 (alerts), #08 (Reminder); `hub_set_brightness/volume` chưa có ticket hub-core

| Hợp đồng | UiState / UserInput |
|---|---|
| `AlertData.kind` / `.serial` | |
| `.title` / `.time` / `.body` | |
| `dismiss()` | |
| `VoiceSlider.kind` / `.value` / `.serial` | |

## Answer

Thêm `contract/alert.slint`, `overlays/{alert,voice-slider,confirm}.slint`; Alert và VoiceSliderOverlay gắn vào Shell trước Orb (dock vẫn nổi trên báo); Orb nhấp nháy khi có báo; Setting dùng `Confirm`. Kịch bản demo trong `src/ui/demo/alerts.rs`. Hàng đợi/tự tắt do hub-core; vòng lan trên Pi chờ kiểm tay.

## Comments

- 2026-11-23 (review hub-ui): Đóng báo có hiệu ứng: hẹn giờ mờ đi, Reminder thu nhỏ về góc trái dưới (phía "Đã nhắc gần đây"); nội dung giữ lại trong lúc mờ. Còn lại cho người (ready-for-human): kiểm bằng mắt theo kịch bản demo, đo fps trên Pi, rồi tick nghiệm thu.
