---
status: accepted
---

# Thiết bị đích là máy tính bảng Debian 13 x86_64, build ngay trên máy

Hub chạy trên máy tính bảng Fujitsu Q5010 (x86_64) cài Debian 13 trixie không desktop, thay cho Raspberry Pi 4. Màn cảm ứng, mic và loa đều tích hợp. Binary build ngay trên máy tính bảng bằng `scripts/install-linux.sh` (feature `kiosk`), không cross-compile từ Mac: Mac là Apple Silicon nên build amd64 trong Docker phải giả lập, chậm; máy tính bảng tự build được với thư viện hệ thống của chính nó. Thay ADR 0011.

## Consequences

- Feature `pi` đổi tên thành `kiosk` (vẫn `backend-linuxkms` + FemtoVG, ADR 0003).
- Audio dùng thiết bị ALSA mặc định; bỏ yêu cầu XVF3800. Không có AEC phần cứng (ADR 0005).
- Bỏ `docker/pi-build.Dockerfile`, `scripts/build-pi.sh`, `scripts/deploy-pi.sh`.
- Thư mục `target/kiosk` riêng cho bản kiosk.
