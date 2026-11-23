mod audio;
pub(crate) mod effects;
mod jobs;
mod tick;
mod tools;
mod voice;

use crate::{
    Io, Ports, Store,
    conversation::{Conversation, Input},
};
use std::time::Duration;
use tokio::time;
use voice_client::VoiceEvent;

/// Runs the hub actor until the UI side drops its receiver.
pub async fn run(mut store: Store, ports: Ports, mut io: Io) -> anyhow::Result<()> {
    let mut timers = crate::features::timers::Feature::load(&store)?;
    let mut countdowns = crate::features::countdowns::Feature::load(&store)?;
    let mut conversation = Conversation::default();
    let mut jobs = jobs::Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    for effect in timers
        .snapshot()
        .into_iter()
        .chain(countdowns.snapshot(ports.clock.now().date()))
    {
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
                let mut pending = tick::tick(ports.clock.as_ref());
                pending.extend(conversation.on(Input::Tick, ports.clock.now()));
                pending.extend(countdowns.snapshot(ports.clock.now().date()));
                pending.extend(timers.tick(ports.clock.now().timestamp().as_second()));
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            event = jobs.next() => {
                match event {
                    jobs::Internal::Timeout(Some(call)) => effects::execute(effects::Effect::ReplyError(call, "Job timed out"), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?,
                    jobs::Internal::Timeout(None) => {},
                    jobs::Internal::JobDone(_id, _result, _reply) => {},
                }
            }
            Some(event) = io.voice_rx.recv() => {
                let pending = match event {
                    VoiceEvent::ToolCall(call) => tools::route(&mut timers, &mut countdowns, call, ports.clock.now()),
                    event => voice::on(event, &mut conversation, ports.clock.now()),
                };
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(event) = io.audio_rx.recv() => {
                for effect in audio::on(event, &mut conversation, ports.clock.now()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(_) = io.input_rx.recv() => {}
            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
