use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

use crate::{
    OpusFrame, VoiceCommand, VoiceEvent,
    connection::{ClientIo, ConnectionEnd, send_with_timeout},
};

pub(super) async fn run(
    mut socket: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    session_id: String,
    io: ClientIo<'_>,
) -> ConnectionEnd {
    loop {
        tokio::select! {
            frame = socket.next() => match frame {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return ConnectionEnd::Failed { became_ready: true },
                Some(Ok(Message::Text(text))) => {
                    if let Some(event) = parse_text_event(&text) {
                        match event {
                            VoiceEvent::TtsAudio(_) => {
                                let _ = io.events.try_send(event);
                            }
                            event => if io.events.send(event).await.is_err() {
                                return ConnectionEnd::Stopped;
                            },
                        }
                    } else {
                        log::debug!("ignoring unknown or malformed voice downlink text");
                    }
                }
                Some(Ok(Message::Binary(data))) => {
                    let _ = io.events.try_send(VoiceEvent::TtsAudio(OpusFrame(data.to_vec())));
                }
                Some(Ok(_)) => {}
            },
            command = io.commands.recv(), if io.pending_commands.len() < crate::COMMAND_CAPACITY => match command {
                Some(VoiceCommand::ListenStart) => {
                    let text = json!({
                        "type": "listen", "state": "start", "mode": "realtime", "session_id": session_id
                    }).to_string();
                    if send_with_timeout(&mut socket, Message::Text(text.into())).await.is_err() {
                        return ConnectionEnd::Failed { became_ready: true };
                    }
                }
                Some(VoiceCommand::ListenStop) => {
                    let text = json!({"type": "listen", "state": "stop", "session_id": session_id}).to_string();
                    if send_with_timeout(&mut socket, Message::Text(text.into())).await.is_err() {
                        return ConnectionEnd::Failed { became_ready: true };
                    }
                }
                Some(VoiceCommand::Abort) => {
                    let text = json!({"type": "abort", "session_id": session_id}).to_string();
                    if send_with_timeout(&mut socket, Message::Text(text.into())).await.is_err() {
                        return ConnectionEnd::Failed { became_ready: true };
                    }
                }
                Some(VoiceCommand::Audio(OpusFrame(data))) => {
                    if send_with_timeout(&mut socket, Message::Binary(data.into())).await.is_err() {
                        return ConnectionEnd::Failed { became_ready: true };
                    }
                }
                Some(command @ VoiceCommand::DirectSpeech { .. }) => io.pending_commands.push_back(command),
                None => {
                    let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
                    return ConnectionEnd::Stopped;
                }
            },
            _ = io.events.closed() => {
                let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
                return ConnectionEnd::Stopped;
            },
            changed = io.config.changed(), if io.config.has_changed().is_ok() => match changed {
                Err(_) => continue,
                Ok(()) => {
                    let updated = io.config.borrow_and_update().clone();
                    if updated != *io.active_config {
                        let _ = send_with_timeout(&mut socket, Message::Close(None)).await;
                        return ConnectionEnd::Reconfigured(updated);
                    }
                }
            },
        }
    }
}

fn parse_text_event(text: &str) -> Option<VoiceEvent> {
    let value: Value = serde_json::from_str(text).ok()?;
    match value.get("type")?.as_str()? {
        "stt" => Some(VoiceEvent::Stt {
            text: value.get("text")?.as_str()?.to_owned(),
        }),
        "tts" => match value.get("state")?.as_str()? {
            "start" => {
                let origin = match (
                    value.get("source").and_then(Value::as_str),
                    value.get("request_id").and_then(Value::as_str),
                ) {
                    (Some("client_direct"), Some(request_id)) => crate::TtsOrigin::ClientDirect {
                        request_id: request_id.to_owned(),
                    },
                    _ => crate::TtsOrigin::Agent,
                };
                Some(VoiceEvent::TtsStart { origin })
            }
            "sentence_start" => Some(VoiceEvent::TtsText {
                text: value.get("text")?.as_str()?.to_owned(),
            }),
            "stop" => Some(VoiceEvent::TtsStop {
                reason: crate::TtsStopReason::Completed,
            }),
            _ => None,
        },
        _ => None,
    }
}
