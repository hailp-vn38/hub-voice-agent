use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};
use url::Url;

pub(super) async fn ota_for(websocket_url: &str) -> Url {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let websocket_url = websocket_url.to_owned();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 2048];
        let _ = stream.read(&mut request).await.unwrap();
        let body = serde_json::json!({ "websocket": { "url": websocket_url, "token": "token" } })
            .to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    Url::parse(&format!("http://{address}/voice/ota/")).unwrap()
}

pub(super) async fn failing_ota() -> (Url, mpsc::UnboundedReceiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (requests_tx, requests_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut request = vec![0; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let _ = requests_tx.send(());
            stream
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        }
    });
    (
        Url::parse(&format!("http://{address}/voice/ota/")).unwrap(),
        requests_rx,
    )
}

/// OTA that answers every request with the same WebSocket URL.
pub(super) async fn ota_for_every(websocket_url: &str) -> Url {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let body =
        serde_json::json!({ "websocket": { "url": websocket_url, "token": "token" } }).to_string();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut request = vec![0; 2048];
            let _ = stream.read(&mut request).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    Url::parse(&format!("http://{address}/voice/ota/")).unwrap()
}
