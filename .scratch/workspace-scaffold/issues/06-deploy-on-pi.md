# 06 — Deploy và chạy trên Pi thật

Status: ready-for-human
Blocked by: 03, 05
Spec: [../spec.md](../spec.md)

Cần Pi 4 + màn 5 inch thật, nên là việc của người. Agent có thể viết sẵn hai file dưới đây; phần chạy và nghiệm thu do người làm.

## Việc

- `deploy/ai-hub.service`: `User=hub`, `SupplementaryGroups=video input render audio`, `Restart=always`, `StateDirectory=ai-hub`, `ExecStart=/opt/ai-hub/ai-hub`.
- `scripts/deploy-pi.sh <host>`: `rsync` binary + service sang Pi, `systemctl daemon-reload` và `restart ai-hub`. Lần đầu (cờ `--init`): cài `libinput10 libgbm1 libegl1 libgles2 libfontconfig1`, tạo user `hub`, `enable` service.
- Trên Pi: Raspberry Pi OS Lite (bookworm, arm64), không desktop.

## Nghiệm thu (người làm)

- [ ] `scripts/deploy-pi.sh --init <host>` rồi `scripts/deploy-pi.sh <host>` chạy được.
- [ ] Màn hình hiện đồng hồ toàn màn qua DRM/KMS, chạy theo giây.
- [ ] `systemctl kill ai-hub` → service tự khởi động lại.
- [ ] Reboot Pi → app tự lên.
- [ ] Ghi vào phần Comments: mode màn hình thực tế, hướng xoay có cần chỉnh không, thời gian từ bật nguồn tới thấy đồng hồ, cảnh báo/lỗi trong `journalctl -u ai-hub`.

## Comments
