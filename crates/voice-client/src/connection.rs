//! One WebSocket connection: request, hello exchange, then the Ready voice loop.

use std::{fmt, future::Future, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    sync::{mpsc, watch},
    time::Instant,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Message,
        client::IntoClientRequest,
        handshake::client::Request,
        http::{HeaderValue, header::AUTHORIZATION},
    },
};
use url::Url;

use crate::{
    DeviceIdentity, VoiceCommand, VoiceConfig, VoiceEvent,
    protocol::{ClientHello, ServerHello},
};

mod timeouts;
mod voice;

pub(crate) use timeouts::{Timeouts, bounded};

#[cfg(test)]
#[path = "connection_tests.rs"]
pub(crate) mod tests;
#[cfg(test)]
#[path = "connection/timeout_tests.rs"]
mod timeout_tests;

pub(crate) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Why a connection attempt, or a Ready connection, ended. The reconnect loop logs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    OtaRequest,
    OtaResponse,
    /// The URL or a header value from OTA or `DeviceIdentity` is unusable.
    InvalidRequest,
    Connect,
    TimedOut,
    Write,
    BadHello,
    Closed,
    Idle,
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::OtaRequest => "OTA request failed",
            Self::OtaResponse => "OTA response has no usable websocket url and token",
            Self::InvalidRequest => "websocket url or identity header is invalid",
            Self::Connect => "websocket connect failed",
            Self::TimedOut => "server did not answer in time",
            Self::Write => "websocket write failed or timed out",
            Self::BadHello => "server hello missing or invalid",
            Self::Closed => "websocket closed",
            Self::Idle => "no frame from server within the idle timeout",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Ready,
    Offline { retry_in: Duration },
    Enrolling,
}

pub(crate) enum ConnectionEnd {
    Failed {
        cause: Failure,
        /// Whether the connection stayed Ready for `Timeouts::stable_after`.
        was_stable: bool,
    },
    Reconfigured(Box<VoiceConfig>),
    RetryEnrollment,
    Stopped,
}

impl ConnectionEnd {
    pub(crate) fn failed(cause: Failure) -> Self {
        Self::Failed {
            cause,
            was_stable: false,
        }
    }
}

pub(crate) async fn send_with_timeout<S>(sink: &mut S, message: Message) -> Result<(), Failure>
where
    S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    tokio::time::timeout(WRITE_TIMEOUT, sink.send(message))
        .await
        .map_err(|_| Failure::Write)?
        .map_err(|_| Failure::Write)
}

/// Everything the client task owns across reconnects.
pub(crate) struct Client {
    pub events: mpsc::Sender<VoiceEvent>,
    pub commands: mpsc::Receiver<VoiceCommand>,
    pub config: watch::Receiver<VoiceConfig>,
    pub active_config: VoiceConfig,
    pub timeouts: Timeouts,
}

impl Client {
    pub(crate) fn new(
        mut config: watch::Receiver<VoiceConfig>,
        commands: mpsc::Receiver<VoiceCommand>,
        events: mpsc::Sender<VoiceEvent>,
        timeouts: Timeouts,
    ) -> Self {
        let active_config = config.borrow_and_update().clone();
        Self {
            events,
            commands,
            config,
            active_config,
            timeouts,
        }
    }

    pub(crate) async fn emit(&self, event: VoiceEvent) -> Result<(), ConnectionEnd> {
        self.events
            .send(event)
            .await
            .map_err(|_| ConnectionEnd::Stopped)
    }

    /// The watched config, if it now differs from the active one.
    fn changed_config(&mut self) -> Option<VoiceConfig> {
        let updated = self.config.borrow_and_update().clone();
        (updated != self.active_config).then_some(updated)
    }

    /// Runs `future` while not Ready: commands that need a session are refused
    /// or dropped, and a config change or shutdown ends the wait early.
    pub(crate) async fn operation<F, T>(&mut self, future: F) -> Result<T, ConnectionEnd>
    where
        F: Future<Output = Result<T, Failure>>,
    {
        self.operation_inner(future, false).await
    }

    pub(crate) async fn wait_for_enrollment(&mut self) -> ConnectionEnd {
        match self
            .operation_inner(std::future::pending::<Result<(), Failure>>(), true)
            .await
        {
            Err(end) => end,
            Ok(()) => unreachable!(),
        }
    }

    async fn operation_inner<F, T>(
        &mut self,
        future: F,
        enrolling: bool,
    ) -> Result<T, ConnectionEnd>
    where
        F: Future<Output = Result<T, Failure>>,
    {
        tokio::pin!(future);
        loop {
            tokio::select! {
                biased;
                command = self.commands.recv() => match command {
                    Some(VoiceCommand::Connect) if enrolling => return Err(ConnectionEnd::RetryEnrollment),
                    Some(VoiceCommand::Connect) => {},
                    Some(VoiceCommand::DirectSpeech { request_id, .. }) => {
                        self.emit(VoiceEvent::DirectSpeechRejected {
                            request_id,
                            reason: crate::DirectSpeechError::Offline,
                        })
                        .await?;
                    }
                    Some(
                        VoiceCommand::Audio(_)
                        | VoiceCommand::ListenStart
                        | VoiceCommand::ListenStop
                        | VoiceCommand::Abort,
                    ) => {}
                    None => return Err(ConnectionEnd::Stopped),
                },
                _ = self.events.closed() => return Err(ConnectionEnd::Stopped),
                changed = self.config.changed(), if self.config.has_changed().is_ok() => {
                    if let Some(updated) = changed.ok().and_then(|()| self.changed_config()) {
                        return Err(ConnectionEnd::Reconfigured(Box::new(updated)));
                    }
                }
                result = &mut future => return result.map_err(ConnectionEnd::failed),
            }
        }
    }
}

pub(crate) async fn connect_and_wait(
    client: &mut Client,
    url: Url,
    token: String,
) -> ConnectionEnd {
    let request = match websocket_request(&url, &token, &client.active_config.device) {
        Ok(request) => request,
        Err(cause) => return ConnectionEnd::failed(cause),
    };
    let connect = bounded(client.timeouts.handshake, async {
        connect_async(request)
            .await
            .map(|(socket, _)| socket)
            .map_err(|_| Failure::Connect)
    });
    let mut socket = match client.operation(connect).await {
        Ok(socket) => socket,
        Err(end) => return end,
    };
    let session_id = match exchange_hello(client, &mut socket).await {
        Ok(session_id) => session_id,
        Err(end) => return close_unless_failed(&mut socket, end).await,
    };
    if let Err(end) = client
        .emit(VoiceEvent::Connection(ConnectionState::Ready))
        .await
    {
        return close_unless_failed(&mut socket, end).await;
    }

    let ready_at = Instant::now();
    match voice::run(socket, session_id, client).await {
        ConnectionEnd::Failed { cause, .. } => ConnectionEnd::Failed {
            cause,
            was_stable: ready_at.elapsed() >= client.timeouts.stable_after,
        },
        end => end,
    }
}

fn websocket_request(url: &Url, token: &str, device: &DeviceIdentity) -> Result<Request, Failure> {
    let header = |value: &str| HeaderValue::from_str(value).map_err(|_| Failure::InvalidRequest);
    let mut request = url
        .as_str()
        .into_client_request()
        .map_err(|_| Failure::InvalidRequest)?;
    let headers = request.headers_mut();
    headers.insert(AUTHORIZATION, header(&format!("Bearer {token}"))?);
    headers.insert("device-id", header(&device.device_id)?);
    headers.insert("client-id", header(&device.client_id)?);
    Ok(request)
}

/// Sends the ClientHello and returns the session id from a valid ServerHello.
async fn exchange_hello(client: &mut Client, socket: &mut Socket) -> Result<String, ConnectionEnd> {
    let hello = serde_json::to_string(&ClientHello::v1()).expect("ClientHello always serializes");
    client
        .operation(send_with_timeout(socket, Message::Text(hello.into())))
        .await?;
    let limit = client.timeouts.handshake;
    let reply = client
        .operation(bounded(limit, next_data_frame(socket)))
        .await?;
    let Message::Text(text) = reply else {
        return Err(ConnectionEnd::failed(Failure::BadHello));
    };
    match serde_json::from_str::<ServerHello>(&text) {
        Ok(hello) if hello.is_valid() => Ok(hello.session_id().to_owned()),
        _ => Err(ConnectionEnd::failed(Failure::BadHello)),
    }
}

/// The next frame that is not a ping or pong.
async fn next_data_frame(socket: &mut Socket) -> Result<Message, Failure> {
    loop {
        match socket.next().await {
            Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
            Some(Ok(message)) => return Ok(message),
            Some(Err(_)) | None => return Err(Failure::Closed),
        }
    }
}

/// Closes the socket politely when the client leaves on purpose; a failed
/// socket is just dropped.
async fn close_unless_failed(socket: &mut Socket, end: ConnectionEnd) -> ConnectionEnd {
    if !matches!(end, ConnectionEnd::Failed { .. }) {
        let _ = send_with_timeout(socket, Message::Close(None)).await;
    }
    end
}
