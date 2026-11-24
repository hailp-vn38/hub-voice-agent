# Thêm một tính năng mới

Checklist theo thứ tự. Mẫu để sao chép: `photos` (có job mạng, store, màn riêng) hoặc `timers` (chỉ luật + store). Quyết định nền: [ADR 0015](adr/0015-feature-registry-and-trait.md).

Một tính năng gồm một `Screen`, một thư mục `features/<x>/`, một hàng trong registry, và phần UI. Thiếu một nơi nào ở **A** thì compiler hoặc test báo; **B** (Slint) thì chỉ test round-trip `Screen` bắt được phần nối `Screen`.

## A. `hub-core`

1. **`view.rs`**: thêm biến thể `Screen::<X>` và thêm vào `Screen::ALL` (đổi độ dài mảng). Các `match Screen` không có `_` sẽ chỉ chỗ còn thiếu: `runtime/tools.rs::with_feature` (phân loại: tính năng hay màn khung) và `apps/hub/src/ui/bridge/inputs.rs` (hai chiều).
2. **`features/<x>/`** theo khuôn: `mod.rs`, `feature.rs`, `rules.rs`, `store.rs`, `tools.rs`, `tests.rs`. Khai báo `pub(crate) mod <x>;` trong `features/mod.rs`.
3. **Job nền** (nếu có `Effect::Spawn`): khai báo hằng id hoặc tiền tố id trong `features/<x>/mod.rs` (xem `weather::JOB`, `home::JOB_PREFIX`, `calendar::JOB_PREFIXES`) và dùng đúng hằng đó trong `feature.rs`. Không viết chuỗi id rải rác.
4. **`features/dispatch.rs`**: `impl Feature for <x>::Feature` với `handle_tool` (bắt buộc) và `handle_job` (nếu có job). Đây là chỗ duy nhất nối method riêng của tính năng vào interface chung.
5. **`features/registry.rs`**: thêm một `Spec` vào `FEATURES` (tăng độ dài mảng): `screen`, `tool_prefixes`, `server_keywords`, `job_prefixes`, `definitions`. Từ hàng này suy ra: định tuyến tool `hub_*`, ánh xạ tool server → màn, định tuyến job, và `tool_definitions()`. Test `every_defined_tool_routes_back_to_its_own_feature` bắt tool có tên lệch tiền tố.
6. **`runtime/hub.rs`**: thêm field `pub(super) <x>: <x>::Feature`, gọi `load`/`new` trong `Hub::load`, thêm `snapshot()` vào `Hub::snapshot`, và `tick` vào `Hub::tick` nếu cần. Thêm field vào destructure trong `runtime/tools.rs::with_feature` và nhánh `match`.
7. **`ui_state.rs`**: trường dữ liệu trong `UiState`.
8. **`present/<x>.rs`**: kiểu view; thêm vào `HubView` (`present/mod.rs`) và `MenuView` (`present/menu.rs`); re-export trong `lib.rs`.
9. **`store/migrations/`**: file SQL mới đánh số tiếp, nếu có dữ liệu lưu.
10. **Chỉ khi cần**: `runtime/data.rs` (xoá dữ liệu ở Setting, kèm `DataKind`), `runtime/effects.rs` (effect riêng), `runtime/tool_trail.rs` (kết quả effect cho dải bước), `UserInput` trong `lib.rs` (thao tác chạm).
11. **Test**: luật trong `features/<x>/tests.rs` hoặc `rules_tests.rs`; luồng qua interface của crate trong `crates/hub-core/tests/<x>.rs` (xem `tests/support`).

## B. `apps/hub`

1. `ui/contract/<x>.slint` (struct + `global <X>Data`) và export trong `ui/app.slint`.
2. `ui/contract/nav.slint`: thêm biến thể vào `enum Screen`; `ui/contract/menu.slint` và `ui/screens/menu.slint`: ô mới.
3. `ui/screens/<x>.slint` (một màn một file, dưới 300 dòng); nối vào `ui/components/shell.slint` và `ui/components/screen-fade.slint`; thêm icon vào `ui/components/icon.slint`.
4. `src/ui/bridge/<x>.rs` (`apply_<x>` và callback), khai báo trong `bridge/mod.rs`, thêm `changes.field(|view| &view.<x>, <x>::apply_<x>)` trong `bridge/update.rs`.
5. `bridge/inputs.rs`: hai nhánh `to_core_screen` / `to_ui_screen`. Test `every_hub_core_screen_round_trips_through_slint` duyệt `Screen::ALL` nên tự bắt thiếu.

## C. Tài liệu

- `GLOSSARY.md`: thêm tên tính năng vào mục **Màn tính năng**.
- `docs/codebase.md`: dòng mới trong bảng module (§3.2) và cây thư mục.
- Quyết định đáng nhớ thì viết ADR mới.

## Kiểm tra cuối

```bash
cargo test --workspace
```
