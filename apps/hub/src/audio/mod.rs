mod devices;
mod mixer;
mod sounds;

use anyhow::{Context, Result, anyhow};
use hub_core::{AudioCommand, AudioEvent};
use rtrb::{Producer, RingBuffer};
use std::{
    thread,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, watch};

const PCM_RING_SAMPLES: usize = 4096;
const WORKER_CHUNK_SAMPLES: usize = 256;

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioConfig;

#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)] // Public fields are consumed by the capture and Orb tickets.
pub struct Levels {
    pub mic: f32,
    pub tts: f32,
}

pub struct AudioHandle {
    pub commands: mpsc::Sender<AudioCommand>,
    pub events: mpsc::Receiver<AudioEvent>,
    pub levels: watch::Receiver<Levels>,
}

pub fn spawn(_config: AudioConfig) -> Result<AudioHandle> {
    let (commands, command_rx) = mpsc::channel(32);
    let (event_tx, events) = mpsc::channel(16);
    let (level_tx, levels) = watch::channel(Levels::default());
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);

    let _thread = thread::Builder::new()
        .name("hub-audio".into())
        .spawn(move || {
            let setup = (|| -> Result<_> {
                let (pcm_tx, pcm_rx) = RingBuffer::new(PCM_RING_SAMPLES);
                let (stream, rate) = devices::open_default_output(pcm_rx)?;
                let mixer = mixer::Mixer::new(sounds::Sounds::load(rate)?, rate);
                devices::start(&stream)?;
                Ok((pcm_tx, stream, rate, mixer))
            })();

            let (pcm_tx, stream, _rate, mut mixer) = match setup {
                Ok(setup) => {
                    if ready_tx.send(Ok(())).is_err() {
                        return;
                    }
                    setup
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error.to_string()));
                    return;
                }
            };
            run_worker(command_rx, event_tx, level_tx, pcm_tx, &mut mixer, stream);
        })
        .context("tạo audio thread")?;

    match ready_rx
        .recv()
        .context("audio thread dừng trước khi sẵn sàng")?
    {
        Ok(()) => Ok(AudioHandle {
            commands,
            events,
            levels,
        }),
        Err(error) => Err(anyhow!("khởi tạo audio output: {error}")),
    }
}

fn run_worker(
    mut commands: mpsc::Receiver<AudioCommand>,
    _events: mpsc::Sender<AudioEvent>,
    levels: watch::Sender<Levels>,
    mut pcm: Producer<f32>,
    mixer: &mut mixer::Mixer,
    _stream: cpal::Stream,
) {
    let mut output = [0.0; WORKER_CHUNK_SAMPLES];
    let mut updated_at = Instant::now();

    loop {
        loop {
            match commands.try_recv() {
                Ok(command) => mixer.apply(command),
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => return,
            }
        }

        let count = pcm.slots().min(output.len());
        if count > 0 {
            mixer.pull(&mut output[..count]);
            let _ = pcm.push_entire_slice(&output[..count]);
        } else {
            thread::sleep(Duration::from_millis(2));
        }

        if updated_at.elapsed() >= Duration::from_millis(33) {
            levels.send_replace(Levels {
                mic: 0.0,
                tts: mixer.level(),
            });
            updated_at = Instant::now();
        }
    }
}
