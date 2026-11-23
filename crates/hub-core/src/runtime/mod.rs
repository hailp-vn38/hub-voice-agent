mod effects;
mod jobs;
mod tick;

use std::time::Duration;

use tokio::time;

use crate::{Io, Ports};

/// Runs the hub actor until the UI side drops its receiver.
pub async fn run(ports: Ports, mut io: Io) -> anyhow::Result<()> {
    let mut jobs = jobs::Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                for effect in tick::tick(ports.clock.as_ref()) {
                    effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await;
                }
            }
            event = jobs.next() => {
                match event {
                    jobs::Internal::Timeout(Some(call)) => effects::execute(effects::Effect::ReplyError(call, "Job timed out"), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await,
                    jobs::Internal::Timeout(None) => {},
                    jobs::Internal::JobDone(_id, _result, _reply) => {},
                }
            }
            Some(_) = io.voice_rx.recv() => {}
            Some(_) = io.input_rx.recv() => {}
            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
