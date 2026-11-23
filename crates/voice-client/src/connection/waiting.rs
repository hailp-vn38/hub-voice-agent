//! Waiting while not Ready: OTA calls, WebSocket connect, backoff and
//! enrollment all run here, so commands are handled the same way in each.

use std::{future::Future, time::Duration};

use super::{Client, ConnectionEnd, Failure};
use crate::{DirectSpeechError, VoiceCommand, VoiceEvent};

/// What `VoiceCommand::Connect` means while waiting.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OnConnect {
    Ignore,
    RetryEnrollment,
}

impl Client {
    /// Runs `future` while not Ready: commands that need a session are refused
    /// or dropped, and a config change or shutdown ends the wait early.
    pub(crate) async fn operation<F, T>(&mut self, future: F) -> Result<T, ConnectionEnd>
    where
        F: Future<Output = Result<T, Failure>>,
    {
        self.wait(future, OnConnect::Ignore).await
    }

    /// Waits in `Enrolling` until the user presses Connect or the activation
    /// code expires; either way the caller calls OTA again.
    pub(crate) async fn wait_for_enrollment(
        &mut self,
        expires_in: Option<Duration>,
    ) -> ConnectionEnd {
        let expired = async {
            match expires_in {
                Some(duration) => tokio::time::sleep(duration).await,
                None => std::future::pending().await,
            }
            Ok(())
        };
        match self.wait(expired, OnConnect::RetryEnrollment).await {
            Ok(()) => ConnectionEnd::RetryEnrollment,
            Err(end) => end,
        }
    }

    async fn wait<F, T>(&mut self, future: F, on_connect: OnConnect) -> Result<T, ConnectionEnd>
    where
        F: Future<Output = Result<T, Failure>>,
    {
        tokio::pin!(future);
        loop {
            tokio::select! {
                biased;
                command = self.commands.recv() => match command {
                    Some(VoiceCommand::Connect) if on_connect == OnConnect::RetryEnrollment => {
                        return Err(ConnectionEnd::RetryEnrollment);
                    }
                    Some(VoiceCommand::DirectSpeech { request_id, .. }) => {
                        self.emit(VoiceEvent::DirectSpeechRejected {
                            request_id,
                            reason: DirectSpeechError::Offline,
                        })
                        .await?;
                    }
                    Some(
                        VoiceCommand::Connect
                        | VoiceCommand::Audio(_)
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
