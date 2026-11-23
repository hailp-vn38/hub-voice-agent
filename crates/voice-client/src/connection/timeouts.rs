//! Limits that turn a silent server into a `Failure` instead of a hang.

use std::{future::Future, time::Duration};

use super::Failure;

/// Time limits for detecting a server that accepted a connection and went silent.
/// `None` disables a limit; paused-clock tests need that because the clock
/// auto-advances while real socket IO is pending.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Timeouts {
    /// One OTA discovery request, including reading the body.
    pub ota: Option<Duration>,
    /// The WebSocket upgrade, and separately the wait for the server hello.
    pub handshake: Option<Duration>,
    /// How often the client pings the socket once Ready.
    pub ping: Option<Duration>,
    /// A Ready socket that delivers no frame (not even a pong) for this long is dead.
    pub idle: Option<Duration>,
    /// A connection must stay Ready this long before its drop resets the backoff,
    /// so a server that sends hello and closes at once still backs off.
    pub stable_after: Duration,
}

impl Timeouts {
    pub(crate) const PRODUCTION: Self = Self {
        ota: Some(Duration::from_secs(10)),
        handshake: Some(Duration::from_secs(10)),
        ping: Some(Duration::from_secs(15)),
        idle: Some(Duration::from_secs(45)),
        stable_after: Duration::from_secs(30),
    };

    #[cfg(test)]
    pub(crate) const NONE: Self = Self {
        ota: None,
        handshake: None,
        ping: None,
        idle: None,
        stable_after: Duration::ZERO,
    };
}

pub(crate) async fn bounded<F, T>(limit: Option<Duration>, future: F) -> Result<T, Failure>
where
    F: Future<Output = Result<T, Failure>>,
{
    match limit {
        Some(limit) => tokio::time::timeout(limit, future)
            .await
            .map_err(|_| Failure::TimedOut)?,
        None => future.await,
    }
}
