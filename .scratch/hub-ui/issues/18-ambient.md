# 18 — Ambient

Status: ready-for-agent
Blocked by: 03
Spec: [../spec.md](../spec.md)
Thiết kế: concept.md §6 "Màn hình" (Ambient), "Hiệu ứng theo trạng thái" (Idle), Orb Idle/Ambient

## Việc

- `screens/ambient.slint`: đồng hồ lớn + nhiệt độ, tông tối và độ tương phản thấp; Orb thở nhỏ và mờ.
- Đồng hồ chuyển số nhẹ (mỗi phút); dịch vị trí chậm theo giờ để tránh lưu ảnh trên màn.
- Chạm bất kỳ đâu → `AmbientData.wake()`; wake word thoát do hub-core đổi `Nav.screen`.
- Lớp báo (ticket 14) và ô mốc "Hôm nay là …" vẫn hiện được trên Ambient.
- Độ sáng đèn nền thật do hub-core qua `System`; UI chỉ làm tối nội dung.
- `contract/ambient.slint`: `global AmbientData { in property <string> time; in property <string> temp; in property <string> icon; in property <bool> night; callback wake(); }`.

## Demo

Kịch bản: "Vào Ambient", "Ambient ban đêm", "Chạm để thoát", "Hết giờ khi đang Ambient".

## Nghiệm thu

- [ ] Vào/ra Ambient bằng fade, không giật.
- [ ] Đọc được giờ từ xa trong phòng tối.
- [ ] Chạm thoát; báo hiện trên Ambient.

## Map với hub-core

Map: chờ hub-core #05 (view: Ambient), #09 (nhiệt độ)

| Hợp đồng | UiState / UserInput |
|---|---|
| `AmbientData.time` / `.temp` / `.icon` | |
| `.night` | |
| `wake()` | |
