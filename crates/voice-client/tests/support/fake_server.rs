use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::oneshot};
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
}

impl FakeServer {
    #[allow(clippy::result_large_err)]
    pub async fn start(action: ServerAction) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (received_tx, received) = oneshot::channel();

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
            }
        });

        Self {
            url: format!("ws://{address}"),
            received,
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
