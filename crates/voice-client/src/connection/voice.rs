//! The Ready loop: downlink frames, uplink commands and device tool replies on one socket.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

use crate::{
    OpusFrame, ToolDef, TtsStopReason, VoiceCommand, VoiceEvent,
    connection::{Client, ConnectionEnd, Failure, Socket, send_with_timeout},
    mcp::{self, ToolCalls},
    protocol,
};

#[cfg(test)]
#[path = "voice_tests.rs"]
mod tests;

/// Placeholder period for a disabled limit; its branch is never polled.
const DISABLED: Duration = Duration::from_secs(365 * 24 * 60 * 60);

/// `Err` ends the Ready loop with that outcome.
type Step = Result<(), ConnectionEnd>;

pub(super) async fn run(socket: Socket, session_id: String, client: &mut Client) -> ConnectionEnd {
    let mut session = Session {
        socket,
        session_id,
        tts_active: false,
    };
    let tools = client.active_config.tools.clone();
    let (tool_calls, mut replies) = ToolCalls::new();
    let timeouts = client.timeouts;
    let ping_every = timeouts.ping.unwrap_or(DISABLED);
    let mut ping = tokio::time::interval_at(Instant::now() + ping_every, ping_every);
    let idle = tokio::time::sleep(timeouts.idle.unwrap_or(DISABLED));
    tokio::pin!(idle);
    loop {
        // A disabled limit keeps its branch off via the `if` guards.
        let step = tokio::select! {
            () = &mut idle, if timeouts.idle.is_some() => Err(ConnectionEnd::failed(Failure::Idle)),
            _ = ping.tick(), if timeouts.ping.is_some() => {
                session.send(Message::Ping(Vec::new().into())).await
            }
            frame = session.socket.next() => {
                if let Some(limit) = timeouts.idle {
                    idle.as_mut().reset(Instant::now() + limit);
                }
                session.on_frame(frame, client, &tools, &tool_calls).await
            }
            Some(reply) = replies.recv() => {
                let frame = protocol::mcp(&session.session_id, reply);
                session.send_text(frame).await
            }
            command = client.commands.recv() => session.on_command(command, client).await,
            _ = client.events.closed() => Err(ConnectionEnd::Stopped),
            changed = client.config.changed(), if client.config.has_changed().is_ok() => {
                match changed.ok().and_then(|()| client.changed_config()) {
                    Some(updated) => Err(ConnectionEnd::Reconfigured(Box::new(updated))),
                    None => Ok(()),
                }
            }
        };
        if let Err(end) = step {
            return session.finish(end, client).await;
        }
    }
}

struct Session {
    socket: Socket,
    session_id: String,
    /// A `TtsStart` was emitted without its `TtsStop` yet.
    tts_active: bool,
}

impl Session {
    async fn send(&mut self, message: Message) -> Step {
        send_with_timeout(&mut self.socket, message)
            .await
            .map_err(ConnectionEnd::failed)
    }

    async fn send_text(&mut self, frame: Value) -> Step {
        self.send(Message::Text(frame.to_string().into())).await
    }

    /// Closes the TTS turn the caller saw start, so every `TtsStart` gets a `TtsStop`.
    async fn interrupt(&mut self, client: &Client) {
        if std::mem::take(&mut self.tts_active) {
            let _ = client
                .emit(VoiceEvent::TtsStop {
                    reason: TtsStopReason::Interrupted,
                })
                .await;
        }
    }

    /// Leaves the loop: a deliberate exit closes the socket, and any open TTS
    /// turn is interrupted before the caller sees the connection change.
    async fn finish(mut self, end: ConnectionEnd, client: &Client) -> ConnectionEnd {
        if !matches!(end, ConnectionEnd::Failed { .. }) {
            let _ = self.send(Message::Close(None)).await;
        }
        self.interrupt(client).await;
        end
    }

    async fn on_frame(
        &mut self,
        frame: Option<Result<Message, WsError>>,
        client: &Client,
        tools: &[ToolDef],
        tool_calls: &ToolCalls,
    ) -> Step {
        match frame {
            Some(Ok(Message::Text(text))) => self.on_text(&text, client, tools, tool_calls).await,
            Some(Ok(Message::Binary(data))) => {
                // Audio is dropped rather than stalling the socket when the caller lags.
                let _ = client
                    .events
                    .try_send(VoiceEvent::TtsAudio(OpusFrame(data.to_vec())));
                Ok(())
            }
            Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                Err(ConnectionEnd::failed(Failure::Closed))
            }
            Some(Ok(_)) => Ok(()),
        }
    }

    async fn on_text(
        &mut self,
        text: &str,
        client: &Client,
        tools: &[ToolDef],
        tool_calls: &ToolCalls,
    ) -> Step {
        let Ok(value) = serde_json::from_str::<Value>(text) else {
            log::debug!("ignoring malformed voice downlink text");
            return Ok(());
        };
        if let Some(payload) = mcp::mcp_payload(&value) {
            return match mcp::handle(payload, tools, &client.events, tool_calls).await {
                Some(response) => {
                    let frame = protocol::mcp(&self.session_id, response);
                    self.send_text(frame).await
                }
                None => Ok(()),
            };
        }
        let Some(event) = protocol::downlink_event(&value) else {
            log::debug!("ignoring unknown voice downlink text");
            return Ok(());
        };
        match event {
            VoiceEvent::TtsStart { .. } => {
                self.interrupt(client).await;
                self.tts_active = true;
                client.emit(event).await
            }
            VoiceEvent::TtsStop { .. } if self.tts_active => {
                self.tts_active = false;
                client.emit(event).await
            }
            VoiceEvent::TtsStop { .. } => Ok(()),
            event => client.emit(event).await,
        }
    }

    async fn on_command(&mut self, command: Option<VoiceCommand>, client: &Client) -> Step {
        let Some(command) = command else {
            return Err(ConnectionEnd::Stopped);
        };
        let id = &self.session_id;
        let frame = match command {
            VoiceCommand::ListenStart => protocol::listen_start(id),
            VoiceCommand::ListenStop => protocol::listen_stop(id),
            VoiceCommand::Abort => protocol::abort(id),
            VoiceCommand::Audio(OpusFrame(data)) => {
                return self.send(Message::Binary(data.into())).await;
            }
            VoiceCommand::DirectSpeech { request_id, text } => {
                if let Err(reason) = protocol::validate_direct_speech(&request_id, &text) {
                    return client
                        .emit(VoiceEvent::DirectSpeechRejected { request_id, reason })
                        .await;
                }
                protocol::direct_speech(id, &request_id, &text)
            }
        };
        self.send_text(frame).await
    }
}
