use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        Message, Utf8Bytes,
        handshake::server::{Request, Response},
        http::HeaderValue,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};

pub enum ServerAction {
    SendHello,
    CloseProtocolError,
    HoldOpen,
}

pub struct ReceivedHandshake {
    pub authorization: String,
    pub device_id: String,
    pub client_id: String,
    pub hello: Value,
}

pub struct FakeServer {
    pub url: String,
    pub received: oneshot::Receiver<ReceivedHandshake>,
    pub closed: oneshot::Receiver<()>,
}

pub struct ReceivedOtaRequest {
    pub method: String,
    pub device_id: String,
    pub client_id: String,
    pub body: String,
}

pub struct FakeOtaServer {
    pub url: String,
    pub received: oneshot::Receiver<ReceivedOtaRequest>,
}

impl FakeOtaServer {
    pub async fn start(status: u16, body: Value) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (received_tx, received) = oneshot::channel();
        let body = body.to_string();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let count = stream.read(&mut request).await.unwrap();
            let request = String::from_utf8_lossy(&request[..count]);
            let mut lines = request.lines();
            let method = lines
                .next()
                .and_then(|line| line.split_whitespace().next())
                .unwrap()
                .to_owned();
            let header = |name: &str| {
                request
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case(name)
                            .then(|| value.trim().to_owned())
                    })
                    .unwrap()
            };
            let device_id = header("device-id");
            let client_id = header("client-id");
            let request_body = request
                .split_once("\r\n\r\n")
                .map_or("", |(_, body)| body)
                .to_owned();
            let _ = received_tx.send(ReceivedOtaRequest {
                method,
                device_id,
                client_id,
                body: request_body,
            });
            let reason = if status < 400 { "OK" } else { "Forbidden" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        Self {
            url: format!("http://{address}/voice/ota/"),
            received,
        }
    }
}

impl FakeServer {
    #[allow(clippy::result_large_err)]
    pub async fn start(action: ServerAction) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (received_tx, received) = oneshot::channel();
        let (closed_tx, closed) = oneshot::channel();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (header_tx, header_rx) = oneshot::channel();
            let socket = accept_hdr_async(stream, move |request: &Request, response: Response| {
                let _ = header_tx.send((
                    header(request, "authorization"),
                    header(request, "device-id"),
                    header(request, "client-id"),
                ));
                Ok(response)
            })
            .await
            .unwrap();
            let (authorization, device_id, client_id) = header_rx.await.unwrap();
            let (mut sink, mut stream) = socket.split();
            let Some(Ok(Message::Text(hello))) = stream.next().await else {
                panic!("expected ClientHello text")
            };
            let hello: Value = serde_json::from_str(&hello).unwrap();
            let _ = received_tx.send(ReceivedHandshake {
                authorization,
                device_id,
                client_id,
                hello,
            });

            match action {
                ServerAction::SendHello => {
                    sink.send(Message::Text(json!({
                        "type": "hello", "transport": "websocket", "session_id": "session-1",
                        "audio_params": { "format": "opus", "sample_rate": 24000, "channels": 1, "frame_duration": 60 }
                    }).to_string().into())).await.unwrap();
                }
                ServerAction::CloseProtocolError => {
                    sink.send(Message::Close(Some(CloseFrame {
                        code: CloseCode::Protocol,
                        reason: Utf8Bytes::from_static("invalid hello"),
                    })))
                    .await
                    .unwrap();
                }
                ServerAction::HoldOpen => {
                    sink.send(Message::Text(json!({
                        "type": "hello", "transport": "websocket", "session_id": "session-1",
                        "audio_params": { "format": "opus", "sample_rate": 24000, "channels": 1, "frame_duration": 60 }
                    }).to_string().into())).await.unwrap();
                    if matches!(stream.next().await, Some(Ok(Message::Close(_)))) {
                        let _ = closed_tx.send(());
                    }
                }
            }
        });

        Self {
            url: format!("ws://{address}"),
            received,
            closed,
        }
    }
}

fn header(request: &Request, name: &str) -> String {
    request
        .headers()
        .get(name)
        .map(HeaderValue::to_str)
        .unwrap()
        .unwrap()
        .to_owned()
}
