use std::future::Future;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::watch;
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

mod voice;

const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Ready,
    Offline { retry_in: Duration },
    Enrolling,
}

pub(super) enum ConnectionEnd {
    Failed { became_ready: bool },
    Reconfigured(crate::VoiceConfig),
    Stopped,
}

pub(super) enum Operation<T> {
    Complete(T),
    Reconfigured(crate::VoiceConfig),
    Stopped,
    Failed,
}

pub(crate) async fn send_with_timeout<S>(sink: &mut S, message: Message) -> Result<(), ()>
where
    S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    tokio::time::timeout(WRITE_TIMEOUT, sink.send(message))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())
}

pub(super) struct ClientIo<'a> {
    pub events: &'a mpsc::Sender<VoiceEvent>,
    pub commands: &'a mut mpsc::Receiver<VoiceCommand>,
    pub pending_commands: &'a mut std::collections::VecDeque<VoiceCommand>,
    pub config: &'a mut watch::Receiver<crate::VoiceConfig>,
    pub active_config: &'a mut crate::VoiceConfig,
}

impl ClientIo<'_> {
    async fn operation<F, T>(&mut self, future: F) -> Operation<T>
    where
        F: Future<Output = Result<T, ()>>,
    {
        await_operation(
            future,
            self.config,
            self.active_config,
            self.commands,
            self.events,
        )
        .await
    }
}

pub(super) async fn await_operation<F, T>(
    future: F,
    config: &mut watch::Receiver<crate::VoiceConfig>,
    active_config: &crate::VoiceConfig,
    commands: &mut mpsc::Receiver<VoiceCommand>,
    events: &mpsc::Sender<VoiceEvent>,
) -> Operation<T>
where
    F: Future<Output = Result<T, ()>>,
{
    tokio::pin!(future);
    loop {
        tokio::select! {
            biased;
            command = commands.recv() => match command {
                Some(VoiceCommand::DirectSpeech { request_id, .. }) => {
                    if events.send(VoiceEvent::DirectSpeechRejected {
                        request_id,
                        reason: crate::DirectSpeechError::Offline,
                    }).await.is_err() {
                        return Operation::Stopped;
                    }
                }
                Some(VoiceCommand::Audio(_) | VoiceCommand::ListenStart | VoiceCommand::ListenStop | VoiceCommand::Abort) => {}
                None => return Operation::Stopped,
            },
            _ = events.closed() => return Operation::Stopped,
            changed = config.changed(), if config.has_changed().is_ok() => match changed {
                Err(_) => continue,
                Ok(()) => {
                    let updated = config.borrow_and_update().clone();
                    if updated != *active_config {
                        return Operation::Reconfigured(updated);
                    }
                }
            },
            result = &mut future => return match result {
                Ok(value) => Operation::Complete(value),
                Err(()) => Operation::Failed,
            },
        }
    }
}

pub(super) async fn connect_and_wait(
    url: Url,
    token: String,
    device: DeviceIdentity,
    mut io: ClientIo<'_>,
) -> ConnectionEnd {
    let Ok(mut request) = url.as_str().into_client_request() else {
        return ConnectionEnd::Failed {
            became_ready: false,
        };
    };
    let headers = request.headers_mut();
    headers.insert(
        AUTHORIZATION,
        match HeaderValue::from_str(&format!("Bearer {token}")) {
            Ok(value) => value,
            Err(_) => {
                return ConnectionEnd::Failed {
                    became_ready: false,
                };
            }
        },
    );
    headers.insert(
        "device-id",
        match HeaderValue::from_str(&device.device_id) {
            Ok(value) => value,
            Err(_) => {
                return ConnectionEnd::Failed {
                    became_ready: false,
                };
            }
        },
    );
    headers.insert(
        "client-id",
        match HeaderValue::from_str(&device.client_id) {
            Ok(value) => value,
            Err(_) => {
                return ConnectionEnd::Failed {
                    became_ready: false,
                };
            }
        },
    );

    let (mut socket, _) = match io
        .operation(async { connect_async(request).await.map_err(|_| ()) })
        .await
    {
        Operation::Complete(connection) => connection,
        Operation::Reconfigured(config) => return ConnectionEnd::Reconfigured(config),
        Operation::Stopped => return ConnectionEnd::Stopped,
        Operation::Failed => {
            return ConnectionEnd::Failed {
                became_ready: false,
            };
        }
    };
    let Ok(hello) = serde_json::to_string(&ClientHello::v1()) else {
        return ConnectionEnd::Failed {
            became_ready: false,
        };
    };
    match io
        .operation(async { send_with_timeout(&mut socket, Message::Text(hello.into())).await })
        .await
    {
        Operation::Complete(()) => {}
        Operation::Reconfigured(config) => {
            let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
            return ConnectionEnd::Reconfigured(config);
        }
        Operation::Stopped => {
            let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
            return ConnectionEnd::Stopped;
        }
        Operation::Failed => {
            return ConnectionEnd::Failed {
                became_ready: false,
            };
        }
    }

    let message = match io
        .operation(async { socket.next().await.ok_or(())?.map_err(|_| ()) })
        .await
    {
        Operation::Complete(Message::Text(message)) => message,
        Operation::Complete(_) | Operation::Failed => {
            return ConnectionEnd::Failed {
                became_ready: false,
            };
        }
        Operation::Reconfigured(config) => {
            let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
            return ConnectionEnd::Reconfigured(config);
        }
        Operation::Stopped => {
            let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
            return ConnectionEnd::Stopped;
        }
    };
    let Ok(server_hello) = serde_json::from_str::<ServerHello>(&message) else {
        return ConnectionEnd::Failed {
            became_ready: false,
        };
    };
    if !server_hello.is_valid() {
        return ConnectionEnd::Failed {
            became_ready: false,
        };
    }
    let session_id = server_hello.session_id().to_owned();
    if io
        .events
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .is_err()
    {
        let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
        return ConnectionEnd::Stopped;
    }

    voice::run(socket, session_id, io).await
}
