mod capture;
mod devices;
mod mixer;
mod sounds;
mod wake;

use anyhow::{Context, Result, anyhow};
use hub_core::{AudioCommand, AudioEvent};
use rtrb::{Consumer, Producer, RingBuffer};
use std::{
    thread,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, watch};

const PCM_RING_SAMPLES: usize = 4096;
const INPUT_PCM_RING_SAMPLES: usize = 16_384;
const WORKER_CHUNK_SAMPLES: usize = 256;

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioConfig;

#[derive(Clone, Copy, Debug, Default)]
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
            let mut capture = capture::Capture::new(wake::load_default());
            let setup = (|| -> Result<_> {
                let (pcm_tx, pcm_rx) = RingBuffer::new(PCM_RING_SAMPLES);
                let (input_pcm_tx, input_pcm_rx) = RingBuffer::new(INPUT_PCM_RING_SAMPLES);
                let (output_stream, rate) = devices::open_output(pcm_rx)?;
                let (input_stream, input_rate) = devices::open_input(input_pcm_tx)?;
                let mixer = mixer::Mixer::new(sounds::Sounds::load(rate)?, rate);
                let resampler = (input_rate != 16_000)
                    .then(|| devices::InputResampler::new(input_rate))
                    .transpose()?;
                devices::start(&output_stream)?;
                devices::start(&input_stream)?;
                Ok((
                    pcm_tx,
                    output_stream,
                    mixer,
                    input_pcm_rx,
                    input_stream,
                    resampler,
                ))
            })();

            let (pcm_tx, output_stream, mut mixer, input_pcm, input_stream, resampler) = match setup
            {
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
            run_worker(
                command_rx,
                event_tx,
                level_tx,
                pcm_tx,
                input_pcm,
                &mut mixer,
                &mut capture,
                resampler,
                output_stream,
                input_stream,
            );
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
        Err(error) => Err(anyhow!("khởi tạo audio: {error}")),
    }
}

fn run_worker(
    mut commands: mpsc::Receiver<AudioCommand>,
    events: mpsc::Sender<AudioEvent>,
    levels: watch::Sender<Levels>,
    mut output_pcm: Producer<f32>,
    mut input_pcm: Consumer<i16>,
    mixer: &mut mixer::Mixer,
    capture: &mut capture::Capture,
    mut input_resampler: Option<devices::InputResampler>,
    _output_stream: cpal::Stream,
    _input_stream: cpal::Stream,
) {
    let mut output = [0.0; WORKER_CHUNK_SAMPLES];
    let mut input = [0; WORKER_CHUNK_SAMPLES];
    let mut updated_at = Instant::now();

    loop {
        loop {
            match commands.try_recv() {
                Ok(command) => {
                    let flush_preroll = matches!(&command, AudioCommand::Mic { on: true });
                    capture.apply(&command);
                    mixer.apply(command);
                    if flush_preroll {
                        for event in capture.push(&[]) {
                            if events.blocking_send(event).is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => return,
            }
        }

        let mut input_count = 0;
        while input_count < input.len() {
            let Ok(sample) = input_pcm.pop() else {
                break;
            };
            input[input_count] = sample;
            input_count += 1;
        }
        if input_count > 0 {
            let captured = match &mut input_resampler {
                Some(resampler) => {
                    let resampled = resampler.push(&input[..input_count]);
                    capture.push(&resampled)
                }
                None => capture.push(&input[..input_count]),
            };
            for event in captured {
                if events.blocking_send(event).is_err() {
                    return;
                }
            }
        }

        let output_count = output_pcm.slots().min(output.len());
        if output_count > 0 {
            mixer.pull(&mut output[..output_count]);
            let _ = output_pcm.push_entire_slice(&output[..output_count]);
        } else {
            thread::sleep(Duration::from_millis(2));
        }

        if updated_at.elapsed() >= Duration::from_millis(33) {
            levels.send_replace(Levels {
                mic: capture.level(),
                tts: mixer.level(),
            });
            updated_at = Instant::now();
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
