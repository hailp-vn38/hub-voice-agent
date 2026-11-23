use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use url::Url;

use crate::{
    ConnectionState, DeviceIdentity, TtsStopReason, VoiceConfig, VoiceEvent, connection::Timeouts,
};

// Real clock: a paused clock auto-advances while socket IO is pending, which
// would fire these limits spuriously.
const SHORT: Timeouts = Timeouts {
    ota: Some(Duration::from_millis(300)),
    handshake: Some(Duration::from_millis(300)),
    ping: Some(Duration::from_millis(50)),
    idle: Some(Duration::from_millis(300)),
    stable_after: Duration::ZERO,
};
const RETRY: Duration = Duration::from_secs(60);

const HELLO: &str = r#"{"type":"hello","transport":"websocket","session_id":"s-1","audio_params":{"format":"opus","sample_rate":24000,"channels":1,"frame_duration":60}}"#;

async fn ota_pointing_to(websocket_url: String) -> Url {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut request = vec![0; 2048];
            let _ = stream.read(&mut request).await;
            let body = serde_json::json!({ "websocket": { "url": websocket_url, "token": "t" } })
                .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    Url::parse(&format!("http://{address}/voice/ota/")).unwrap()
}

/// Accepts TCP connections and holds them open without ever answering.
async fn silent_listener() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            held.push(stream);
        }
    });
    (address.to_string(), handle)
}

/// Completes the WebSocket upgrade, then hands the socket to `script`.
async fn websocket_server<F, Fut>(script: F) -> String
where
    F: Fn(WebSocketStream<TcpStream>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            if let Ok(socket) = accept_async(stream).await {
                tokio::spawn(script(socket));
            }
        }
    });
    format!("ws://{address}/voice/ws/")
}

async fn read_client_hello(socket: &mut WebSocketStream<TcpStream>) {
    while let Some(Ok(message)) = socket.next().await {
        if message.is_text() {
            return;
        }
    }
}

fn start(ota_url: Url) -> tokio::sync::mpsc::Receiver<VoiceEvent> {
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("device-1", "client-1"),
        tools: Vec::new(),
    };
    let (commands, events) = crate::spawn_with(held_config(config), |_| RETRY, SHORT);
    std::mem::forget(commands);
    events
}

async fn next_connection(events: &mut tokio::sync::mpsc::Receiver<VoiceEvent>) -> ConnectionState {
    loop {
        if let Some(VoiceEvent::Connection(state)) = events.recv().await {
            return state;
        }
    }
}

#[tokio::test]
async fn silent_ota_server_times_out_to_offline() {
    let (address, _server) = silent_listener().await;
    let mut events = start(Url::parse(&format!("http://{address}/voice/ota/")).unwrap());

    assert_eq!(
        next_connection(&mut events).await,
        ConnectionState::Connecting
    );
    assert!(matches!(
        next_connection(&mut events).await,
        ConnectionState::Offline { .. }
    ));
}

#[tokio::test]
async fn websocket_upgrade_that_never_answers_times_out_to_offline() {
    let (address, _server) = silent_listener().await;
    let mut events = start(ota_pointing_to(format!("ws://{address}/voice/ws/")).await);

    assert_eq!(
        next_connection(&mut events).await,
        ConnectionState::Connecting
    );
    assert!(matches!(
        next_connection(&mut events).await,
        ConnectionState::Offline { .. }
    ));
}

#[tokio::test]
async fn missing_server_hello_times_out_to_offline() {
    let websocket = websocket_server(|mut socket| async move {
        read_client_hello(&mut socket).await;
        std::future::pending::<()>().await;
    })
    .await;
    let mut events = start(ota_pointing_to(websocket).await);

    assert_eq!(
        next_connection(&mut events).await,
        ConnectionState::Connecting
    );
    assert!(matches!(
        next_connection(&mut events).await,
        ConnectionState::Offline { .. }
    ));
}

#[tokio::test]
async fn ping_before_server_hello_still_reaches_ready() {
    let websocket = websocket_server(|mut socket| async move {
        read_client_hello(&mut socket).await;
        socket.send(Message::Ping(Vec::new().into())).await.unwrap();
        socket.send(Message::Text(HELLO.into())).await.unwrap();
        while socket.next().await.is_some() {}
    })
    .await;
    let mut events = start(ota_pointing_to(websocket).await);

    assert_eq!(
        next_connection(&mut events).await,
        ConnectionState::Connecting
    );
    assert_eq!(next_connection(&mut events).await, ConnectionState::Ready);
}

#[tokio::test]
async fn silent_socket_after_ready_interrupts_tts_then_goes_offline() {
    let websocket = websocket_server(|mut socket| async move {
        read_client_hello(&mut socket).await;
        socket.send(Message::Text(HELLO.into())).await.unwrap();
        socket
            .send(Message::Text(
                r#"{"type":"tts","state":"start","session_id":"s-1"}"#.into(),
            ))
            .await
            .unwrap();
        // Stop reading: pings from the client are never answered.
        std::future::pending::<()>().await;
    })
    .await;
    let mut events = start(ota_pointing_to(websocket).await);

    assert_eq!(
        next_connection(&mut events).await,
        ConnectionState::Connecting
    );
    assert_eq!(next_connection(&mut events).await, ConnectionState::Ready);
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStart { .. })
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStop {
            reason: TtsStopReason::Interrupted
        })
    ));
    assert!(matches!(
        next_connection(&mut events).await,
        ConnectionState::Offline { .. }
    ));
}

#[tokio::test]
async fn quiet_but_healthy_socket_stays_ready() {
    let websocket = websocket_server(|mut socket| async move {
        read_client_hello(&mut socket).await;
        socket.send(Message::Text(HELLO.into())).await.unwrap();
        // Keep polling so tungstenite answers client pings with pongs.
        while socket.next().await.is_some() {}
    })
    .await;
    let mut events = start(ota_pointing_to(websocket).await);

    assert_eq!(
        next_connection(&mut events).await,
        ConnectionState::Connecting
    );
    assert_eq!(next_connection(&mut events).await, ConnectionState::Ready);
    let quiet = tokio::time::timeout(Duration::from_secs(1), events.recv()).await;
    assert!(quiet.is_err(), "unexpected event: {quiet:?}");
}

fn held_config(config: VoiceConfig) -> tokio::sync::watch::Receiver<Option<VoiceConfig>> {
    let (sender, receiver) = tokio::sync::watch::channel(Some(config));
    tokio::spawn(async move {
        sender.closed().await;
    });
    receiver
}
