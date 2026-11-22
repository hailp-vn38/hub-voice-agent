---
status: accepted
---

# Wake word cục bộ hey_jarvis, AEC bằng phần cứng mic array

Wake word chạy trên hub bằng model `hey_jarvis` có sẵn của openWakeWord qua ONNX Runtime (`ort`); đổi từ khoá để sau khi sản phẩm hoàn thiện. Khử vọng (AEC) do ReSpeaker XVF3800 làm, nên loa **phải** cắm vào cổng ra của mic array, không cắm jack của Pi: chip cần tín hiệu tham chiếu thì wake word và barge-in mới hoạt động khi loa đang phát.

## Consequences

- Mọi âm thanh phát ra (TTS, chuông, âm giao diện) đi cùng một đường ra qua mic array.
- Model tiếng Anh: phải chỉnh threshold theo giọng Việt; Setting có độ nhạy thấp / vừa / cao.
- Trên Mac không có AEC: đeo tai nghe, hoặc cắm XVF3800 vào Mac để test thật.
