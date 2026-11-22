---
status: accepted
---

# SQLite cho dữ liệu cục bộ, config là file TOML riêng

Hẹn giờ, Đếm ngược, Reminder, cache thời tiết/lịch/Home Assistant và chỉ mục ảnh lưu trong một file SQLite qua `rusqlite` (feature `bundled`, WAL); file ảnh đã thu nhỏ nằm trong thư mục cạnh đó. Config và bí mật (token server, refresh token Google, token HA, mật khẩu Wi-Fi) là một file TOML riêng, quyền 0600, để Setup web và người vận hành đọc/sửa được trực tiếp.

## Considered Options

- File JSON mỗi store, ghi atomic: đủ cho lượng dữ liệu nhỏ, không thêm C. Bỏ vì muốn transaction khi một thao tác chạm nhiều bảng (vd. Reminder lặp: đánh dấu đã nhắc + lên lịch lần kế) và truy vấn theo khoảng thời gian cho lịch.

## Consequences

- `Store` chỉ có một adapter (SQLite); test dùng SQLite `:memory:`, không viết fake.
- Schema đổi bằng migration đánh số trong `crates/hub-core/src/store/migrations/`.
