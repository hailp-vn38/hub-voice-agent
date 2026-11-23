---
status: accepted
---

# Giao diện theo hệ thiết kế Aurora Glass, token gom ở `Theme`

Mọi màn của hub theo Aurora Glass: tối, kính xanh đậm trong suốt trên ảnh ngôi nhà đêm, thanh điều hướng trái, thanh Voice ở đáy. Ảnh mẫu, asset và bản dựng Slint gốc nằm ở `docs/smart-hub-home`. Màu, cỡ chữ, bo góc và thời gian chuyển động là token trong `ui/theme.slint`; component không viết mã màu hay thời gian riêng. Màn chuyển sang Aurora dần dần, bắt đầu từ Màn trung tâm và Menu tính năng.

Kích thước theo **đơn vị thiết kế** `Theme.unit` = 1 px của canvas 1280 × 600, co đều theo cửa sổ (Shell ghi). Canvas giãn theo tỉ lệ màn: cột phải bám mép phải, hàng ô và thanh Voice bám đáy. Cách này giữ đúng tỉ lệ của bản thiết kế trên 1920 × 1080, 1024 × 600 và 800 × 480 mà không méo.

## Lệch khỏi spec có chủ ý

- **Font**: giữ Be Vietnam Pro thay vì Inter / Noto Sans. Font này hiển thị dấu tiếng Việt tốt và đã đóng gói sẵn. App chỉ có Regular và SemiBold, nên đồng hồ "light" hiện ở nét Regular.
- **Bộ icon**: dùng bộ icon outline bo tròn của `docs/smart-hub-home`. Màn chưa chuyển vẫn dùng Tabler (cũng là outline 2px) cho tới khi được làm lại. Ảnh chưa có icon trong bộ mới nên tạm dùng Tabler.
- **Bố cục**: mới có bố cục ngang có thanh điều hướng trái. Các bố cục compact, vuông (điều hướng ở đáy, lưới 3 cột) và dọc chưa làm.

## Considered Options

- Viết mã màu của bản thiết kế thẳng vào từng component, như bản `docs/smart-hub-home`: nhanh, nhưng mỗi màn lệch nhau một chút. Spec yêu cầu token dùng chung nên bỏ.
- Đổi tên mọi token sang tên Aurora (`bg-primary`, `accent-warm`…): hơn 200 chỗ dùng ở các màn chưa chuyển. Thay vào đó giữ tên cũ, đổi giá trị sang Aurora, ghi tên Aurora trong chú thích.

## Consequences

- Đổi giá trị token cũ (`background`, `card`, `accent`, `warm`, `done`…) làm màu các màn chưa chuyển đổi theo ngay.
- Thang chữ và khoảng cách cũ (`font-*`, `space-*`, `radius`, px tuyệt đối cho 1920 × 1080) còn tồn tại song song với thang Aurora (`type-*`, `radius-*`, theo `unit`), cho tới khi màn cuối cùng chuyển xong.
