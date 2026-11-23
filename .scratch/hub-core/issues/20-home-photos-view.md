# 20 — Nhà và Ảnh: dữ liệu hiển thị và thao tác chạm

Status: ready-for-human
Blocked by: 14
Spec: [../spec.md](../spec.md) · UI: hub-ui #15, #16

## Việc

- Home: đưa `connected` và `configured` (có URL + token HA) ra `UiState`; giữ tập entity đang chờ `call_service` để có `pending`.
- `UserInput::ToggleHome(entity_id)`: light/switch → `turn_on`/`turn_off` qua `HomeSource::call_service`, dùng chung đường với tool.
- `present`: `home` — `kind` từ tiền tố entity (`light.`, `switch.`, còn lại `sensor`), `on`, `value` (giá trị + đơn vị cho sensor), `pending`, `stale`, `updated` "HH:MM".
- Photos: `UserInput::RemovePhoto(id)` xoá bản ghi và tệp ảnh/thumbnail.
- Viewer do hub-core giữ (#14): `viewing` khi `Focus::Photo` có mặt trên màn Ảnh, `viewer-index` là vị trí ảnh focus. Chạm `open(key)` → `UserInput::Focus(Some(Focus::Photo(key)))`; `next`/`previous` → focus ảnh kề (vòng như `hub_image_control`); `close-viewer` → `UserInput::Focus(None)`. `auto-advance` do UI giữ.
- `present`: `photos` — `key`, đường dẫn ảnh và thumbnail (bridge giải mã thành `image`, có cache theo `key`), `caption`, `source`, `time`, `pinned`, `state` (loading khi job tải đang chạy, failed khi lỗi), `title`, `pinned-text`.

## Nghiệm thu

- [x] Test qua `run()` với `HomeSource` giả: `ToggleHome` → `pending` true, rồi `State` mới → `pending` false, `on` đổi.
- [x] Test `present`: `sensor.temp` "23.5" + đơn vị → `value == "23,5 °C"`.
- [x] Test: `hub_image_control` next rồi chạm `close-viewer` → `viewing` false, vẫn ở màn Ảnh.
- [x] Test: `RemovePhoto` → mất khỏi `UiState`, tệp bị xoá; nếu ảnh đang focus thì viewer đóng.

## Comments

- Đã thêm dữ liệu trình bày Nhà/Ảnh, trạng thái tải ảnh và xoá ảnh cùng hai tệp cache. Viewer focus tiếp tục do `hub-core` giữ qua `Focus`; `auto-advance` tiếp tục thuộc UI.
- Gates sau khi ghép `feat/hub-core` ở `ec46e9d`: `scripts/test.sh -p hub-core` (140 passed), `cargo fmt --all -- --check`, `cargo build -p hub` và `git diff --check` đều đạt.
