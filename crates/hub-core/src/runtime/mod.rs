pub(crate) mod effects;
mod jobs;
mod tick;
mod tools;

use std::time::Duration;

use tokio::time;

use crate::{Io, Ports, Store};
use voice_client::VoiceEvent;

/// Runs the hub actor until the UI side drops its receiver.
pub async fn run(mut store: Store, ports: Ports, mut io: Io) -> anyhow::Result<()> {
    let mut timers = crate::features::timers::Feature::load(&store)?;
    let mut jobs = jobs::Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    for effect in timers.snapshot() {
        effects::execute(
            effect,
            &io.ui_tx,
            &io.voice_tx,
            &io.audio_tx,
            &mut jobs,
            &mut store,
        )
        .await?;
    }
    loop {
        tokio::select! {
            _ = tick.tick() => {
                for effect in tick::tick(ports.clock.as_ref()).into_iter().chain(timers.tick(ports.clock.now().timestamp().as_second())) {
                    effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?;
                }
            }
            event = jobs.next() => {
                match event {
                    jobs::Internal::Timeout(Some(call)) => effects::execute(effects::Effect::ReplyError(call, "Job timed out"), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?,
                    jobs::Internal::Timeout(None) => {},
                    jobs::Internal::JobDone(_id, _result, _reply) => {},
                }
            }
            Some(event) = io.voice_rx.recv() => {
                if let VoiceEvent::ToolCall(call) = event {
                    for effect in tools::route(&mut timers, call, ports.clock.now().timestamp().as_second()) {
                        effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?;
                    }
                }
            }
            Some(_) = io.input_rx.recv() => {}
            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
