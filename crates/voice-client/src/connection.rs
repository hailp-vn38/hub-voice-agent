use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message, client::IntoClientRequest, http::HeaderValue, http::header::AUTHORIZATION,
    },
};
use url::Url;

use crate::{
    DeviceIdentity, VoiceCommand, VoiceEvent,
    protocol::{ClientHello, ServerHello},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Ready,
    Offline { retry_in: Duration },
    Enrolling,
}

pub(super) fn spawn_with_websocket(
    url: Url,
    token: String,
    device: DeviceIdentity,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    let (commands_tx, mut commands_rx) = mpsc::channel(64);
    let (events_tx, events_rx) = mpsc::channel(256);
    tokio::spawn(async move {
        let _ = events_tx
            .send(VoiceEvent::Connection(ConnectionState::Connecting))
            .await;
        let result = connect_and_wait(url, token, device, &events_tx, &mut commands_rx).await;
        if result.is_err() {
            let _ = events_tx
                .send(VoiceEvent::Connection(ConnectionState::Offline {
                    retry_in: Duration::from_secs(1),
                }))
                .await;
        }
    });
    (commands_tx, events_rx)
}

async fn connect_and_wait(
    url: Url,
    token: String,
    device: DeviceIdentity,
    events: &mpsc::Sender<VoiceEvent>,
    commands: &mut mpsc::Receiver<VoiceCommand>,
) -> Result<(), ()> {
    let mut request = url.as_str().into_client_request().map_err(|_| ())?;
    let headers = request.headers_mut();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| ())?,
    );
    headers.insert(
        "device-id",
        HeaderValue::from_str(&device.device_id).map_err(|_| ())?,
    );
    headers.insert(
        "client-id",
        HeaderValue::from_str(&device.client_id).map_err(|_| ())?,
    );

    let (mut socket, _) = connect_async(request).await.map_err(|_| ())?;
    let hello = serde_json::to_string(&ClientHello::v1()).map_err(|_| ())?;
    socket
        .send(Message::Text(hello.into()))
        .await
        .map_err(|_| ())?;

    let Some(Ok(Message::Text(message))) = socket.next().await else {
        return Err(());
    };
    let server_hello: ServerHello = serde_json::from_str(&message).map_err(|_| ())?;
    if !server_hello.is_valid() {
        return Err(());
    }
    let _session_id = server_hello.session_id();
    events
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .map_err(|_| ())?;

    loop {
        tokio::select! {
            frame = socket.next() => match frame {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return Err(()),
                Some(Ok(_)) => todo!("server events are implemented by later tickets"),
            },
            command = commands.recv() => match command {
                Some(_) => todo!("voice commands are implemented by later tickets"),
                None => return Ok(()),
            }
        }
    }
}
