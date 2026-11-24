---
status: accepted
---

# Wake word cục bộ hey_jarvis, AEC bằng phần cứng mic array

Wake word chạy trên hub bằng model `hey_jarvis` có sẵn của openWakeWord qua ONNX Runtime (`ort`); đổi từ khoá để sau khi sản phẩm hoàn thiện. Khử vọng (AEC) do ReSpeaker XVF3800 làm, nên loa **phải** cắm vào cổng ra của mic array: chip cần tín hiệu tham chiếu thì wake word và barge-in mới hoạt động khi loa đang phát.

**Cập nhật 2026-11-24:** thiết bị đích đổi sang máy tính bảng Fujitsu Q5010 dùng mic và loa tích hợp (ADR 0014), không còn XVF3800. Phần AEC phần cứng của quyết định này không còn đúng; cách khử vọng trên máy tính bảng chưa được chọn.

## Consequences

- Mọi âm thanh phát ra (TTS, chuông, âm giao diện) đi cùng một đường ra.
- Model tiếng Anh: phải chỉnh threshold theo giọng Việt; Setting có độ nhạy thấp / vừa / cao.
- Trên Mac và máy tính bảng hiện không có AEC: đeo tai nghe khi test barge-in.
