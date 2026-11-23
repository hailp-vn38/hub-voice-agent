//! The client task: discover, connect, and reconnect with backoff.

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    time::Duration,
};

use tokio::sync::{mpsc, watch};

use crate::{
    ConnectionState, VoiceCommand, VoiceConfig, VoiceEvent,
    connection::{self, Client, ConnectionEnd, Timeouts},
    ota,
};

#[cfg(test)]
#[path = "reconnect_tests/mod.rs"]
mod tests;

const COMMAND_CAPACITY: usize = 64;
const EVENT_CAPACITY: usize = 256;
const MIN_BACKOFF: Duration = Duration::from_secs(1);
/// Upper bound on the reconnect delay, applied after jitter.
const MAX_BACKOFF: Duration = Duration::from_secs(60);

pub(crate) fn spawn(
    config: watch::Receiver<Option<VoiceConfig>>,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    spawn_with(config, random_jitter, Timeouts::PRODUCTION)
}

#[cfg(test)]
pub(crate) fn spawn_with_jitter(
    config: watch::Receiver<Option<VoiceConfig>>,
    jitter: impl FnMut(Duration) -> Duration + Send + 'static,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    spawn_with(config, jitter, Timeouts::NONE)
}

pub(crate) fn spawn_with(
    config: watch::Receiver<Option<VoiceConfig>>,
    jitter: impl FnMut(Duration) -> Duration + Send + 'static,
    timeouts: Timeouts,
) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    let (commands_tx, commands_rx) = mpsc::channel(COMMAND_CAPACITY);
    let (events_tx, events_rx) = mpsc::channel(EVENT_CAPACITY);
    let client = Client::new(config, commands_rx, events_tx, timeouts);
    tokio::spawn(client_loop(client, jitter));
    (commands_tx, events_rx)
}

async fn client_loop(mut client: Client, mut jitter: impl FnMut(Duration) -> Duration) {
    let mut backoff = MIN_BACKOFF;
    loop {
        if client.active_config.is_none() {
            if client
                .emit(VoiceEvent::Connection(ConnectionState::Unconfigured))
                .await
                .is_err()
            {
                return;
            }
            match client
                .operation(std::future::pending::<Result<(), connection::Failure>>())
                .await
            {
                Err(ConnectionEnd::Reconfigured(updated)) => {
                    client.active_config = *updated;
                    backoff = MIN_BACKOFF;
                    continue;
                }
                _ => return,
            }
        }
        if client
            .emit(VoiceEvent::Connection(ConnectionState::Connecting))
            .await
            .is_err()
        {
            return;
        }
        let discover = connection::bounded(
            client.timeouts.ota,
            ota::discover(
                client
                    .active_config
                    .as_ref()
                    .expect("configured connection")
                    .ota_url
                    .clone(),
                client
                    .active_config
                    .as_ref()
                    .expect("configured connection")
                    .device
                    .clone(),
            ),
        );
        let end = match client.operation(discover).await {
            Ok(ota::Discovery::Websocket { url, token }) => {
                connection::connect_and_wait(&mut client, url, token).await
            }
            Ok(ota::Discovery::Activation { code, expires_in }) => {
                if client
                    .emit(VoiceEvent::Connection(ConnectionState::Enrolling))
                    .await
                    .is_err()
                    || client.emit(VoiceEvent::Activation { code }).await.is_err()
                {
                    return;
                }
                client.wait_for_enrollment(expires_in).await
            }
            Err(end) => end,
        };
        let cause = match end {
            ConnectionEnd::Stopped => return,
            ConnectionEnd::RetryEnrollment => continue,
            ConnectionEnd::Reconfigured(updated) => {
                client.active_config = *updated;
                backoff = MIN_BACKOFF;
                continue;
            }
            ConnectionEnd::Failed { cause, was_stable } => {
                if was_stable {
                    backoff = MIN_BACKOFF;
                }
                cause
            }
        };
        let retry_in = jitter(backoff).min(MAX_BACKOFF);
        log::warn!("voice connection lost: {cause}; retrying in {retry_in:?}");
        if client
            .emit(VoiceEvent::Connection(ConnectionState::Offline {
                retry_in,
            }))
            .await
            .is_err()
        {
            return;
        }
        backoff = backoff.saturating_mul(2).min(MAX_BACKOFF);
        let wait = async {
            tokio::time::sleep(retry_in).await;
            Ok(())
        };
        match client.operation(wait).await {
            Ok(()) => {}
            Err(ConnectionEnd::Reconfigured(updated)) => {
                client.active_config = *updated;
                backoff = MIN_BACKOFF;
            }
            Err(_) => return,
        }
    }
}

fn random_jitter(nominal: Duration) -> Duration {
    static SEED: std::sync::LazyLock<RandomState> = std::sync::LazyLock::new(RandomState::new);
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut hasher = SEED.build_hasher();
    hasher.write_u64(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let percent = (hasher.finish() % 41) as i128 - 20;
    let nanos = nominal.as_nanos() as i128;
    let jittered = (nanos + nanos * percent / 100).max(0) as u128;
    Duration::from_nanos(jittered.min(u64::MAX as u128) as u64)
}
