//! Hub core: the actor that owns state and drives the UI snapshot. Depends on `voice-client`.

mod ports;
mod ui_state;

use std::{sync::Arc, time::Duration};

pub use ports::{Clock, SystemClock};
use tokio::{sync::watch, time};
pub use ui_state::UiState;

pub struct Ports {
    pub clock: Arc<dyn Clock>,
}

pub struct Io {
    pub ui_tx: watch::Sender<UiState>,
}

/// Runs the hub actor until the UI side drops its receiver.
pub async fn run(ports: Ports, io: Io) -> anyhow::Result<()> {
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let clock = ports.clock.now().strftime("%H:%M:%S").to_string();
                io.ui_tx.send_if_modified(|s| {
                    let changed = s.clock != clock;
                    if changed {
                        s.clock = clock;
                    }
                    changed
                });
            }
            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
