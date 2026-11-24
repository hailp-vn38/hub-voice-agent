---
status: accepted
---

# Registry tính năng và trait `Feature` hẹp

Trước đây một tính năng nằm rải trong sáu nơi không liên quan nhau: bảng tiền tố tool và bảng từ khoá tool server trong `view.rs`, `tool_definitions()` trong `lib.rs`, `match screen` có nhánh `_` trong `runtime/tools.rs`, và chuỗi id job so khớp tay (`"weather"`, `"photos"`, `"home:"`) hai lần trong `runtime/hub.rs`. Quên một nơi không bị compiler bắt.

Quyết định:

- `features/registry.rs` giữ **một bảng** `FEATURES` (`Spec`: màn, tiền tố tool, từ khoá tool server, tiền tố job, `definitions`). Định tuyến tool, tool server, job và `tool_definitions()` đều suy ra từ bảng.
- `features/dispatch.rs` định nghĩa trait `Feature` **hẹp**: `handle_tool` và `handle_job` (mặc định rỗng), cùng `Cx { view, alerts, store, now }`. Mỗi tính năng có một adapter mỏng tới method riêng; chữ ký của `snapshot`, `tick`, `load`, `due`… vẫn tuỳ tính năng và được gọi trực tiếp trên field của `Hub`.
- `runtime/tools.rs::with_feature` chọn tính năng bằng `match Screen` **không có `_`**: thêm biến thể `Screen` mới không compile được cho tới khi được phân loại.
- Job id vẫn là chuỗi (id mang tham số: `home:<entity>`, `calendar-create-<uuid>`), nhưng mỗi tính năng khai báo hằng id/tiền tố của mình và registry trỏ tới đó.

## Rejected

- **Trait đủ cả `snapshot`/`tick`/`load`**: chữ ký khác nhau (`today`, `store`, `alerts`, mạng) nên phải nhồi ngữ cảnh chung vào mọi method, hoặc `Box<dyn Feature>` mất truy cập kiểu cụ thể mà `Hub` cần (`timers.due`, `weather.leave`). Trait hẹp ở đúng chỗ ba nhánh định tuyến lặp lại.
- **`enum JobId`**: id mang tham số, và `Jobs` khoá theo chuỗi để huỷ job cũ cùng khoá; enum thêm mã mà không thêm an toàn đáng kể khi hằng + test routing đã có.
- **`Vec<Box<dyn Feature>>` trong `Hub`**: một seam chỉ có một cách dùng; field cụ thể vẫn cần cho phần ngoài trait.

## Consequences

- Thêm tính năng chạm: thư mục của nó, một hàng registry, một impl trong `dispatch.rs`, một field + một nhánh match trong runtime. Checklist đầy đủ ở [adding-a-feature.md](../adding-a-feature.md).
- Test `every_defined_tool_routes_back_to_its_own_feature` thất bại nếu tool định nghĩa không khớp tiền tố của tính năng.
- Phía Slint vẫn phải khai báo tường minh; `Screen::ALL` làm test round-trip Slint ↔ core tự bao phủ màn mới.
