use anyhow::{Context, Result, anyhow};
use hub_core::{AudioCommand, AudioEvent};
use rtrb::{Consumer, Producer, RingBuffer};
use std::{
    collections::VecDeque,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, watch};

use super::{AudioHandle, Levels, capture, devices, mixer, models, resample, sounds, wake};

/// ~43 ms ở 48 kHz: đủ đệm cho vòng lặp ngủ 5 ms mà không thêm độ trễ thừa.
const PCM_RING_SAMPLES: usize = 2048;
const INPUT_PCM_RING_SAMPLES: usize = 16_384;
const WORKER_CHUNK_SAMPLES: usize = 256;
const IDLE_SLEEP: Duration = Duration::from_millis(5);
const LEVELS_EVERY: Duration = Duration::from_millis(33);
/// ~7.7 s khung mic 60 ms; quá mức này hub-core đã kẹt, bỏ sự kiện cũ nhất.
const MAX_PENDING_EVENTS: usize = 128;

pub fn spawn() -> Result<AudioHandle> {
    let (commands, command_rx) = mpsc::channel(32);
    let (event_tx, events) = mpsc::channel(16);
    let (level_tx, levels) = watch::channel(Levels::default());
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);

    thread::Builder::new()
        .name("hub-audio".into())
        .spawn(move || match Worker::open(command_rx, event_tx, level_tx) {
            Ok(worker) => {
                if ready_tx.send(Ok(())).is_ok() {
                    worker.run();
                }
            }
            Err(error) => {
                let _ = ready_tx.send(Err(format!("{error:#}")));
            }
        })
        .context("tạo audio thread")?;

    ready_rx
        .recv()
        .context("audio thread dừng trước khi sẵn sàng")?
        .map_err(|error| anyhow!("khởi tạo audio: {error}"))?;
    Ok(AudioHandle {
        commands,
        events,
        levels,
    })
}

/// Handle không có thiết bị: lệnh audio bị bỏ, không có sự kiện hay mức âm.
/// Để hub-core và UI vẫn chạy khi máy không có loa/mic.
pub fn disabled() -> AudioHandle {
    let (commands, _) = mpsc::channel(1);
    let (_, events) = mpsc::channel(1);
    let (_, levels) = watch::channel(Levels::default());
    AudioHandle {
        commands,
        events,
        levels,
    }
}

struct Worker {
    commands: mpsc::Receiver<AudioCommand>,
    outbox: Outbox,
    levels: watch::Sender<Levels>,
    output_pcm: Producer<f32>,
    input_pcm: Consumer<i16>,
    mixer: mixer::Mixer,
    capture: capture::Capture,
    /// Có khi model wake word đang được tải; nhận thư mục model khi tải xong.
    wake_models: Option<std::sync::mpsc::Receiver<std::path::PathBuf>>,
    input_resampler: Option<resample::InputResampler>,
    _streams: [cpal::Stream; 2],
}

impl Worker {
    fn open(
        commands: mpsc::Receiver<AudioCommand>,
        events: mpsc::Sender<AudioEvent>,
        levels: watch::Sender<Levels>,
    ) -> Result<Self> {
        let (output_pcm, pcm_rx) = RingBuffer::new(PCM_RING_SAMPLES);
        let (input_pcm_tx, input_pcm) = RingBuffer::new(INPUT_PCM_RING_SAMPLES);
        let (output_stream, rate) = devices::open_output(pcm_rx)?;
        let (input_stream, input_rate) = devices::open_input(input_pcm_tx)?;
        let mixer = mixer::Mixer::new(sounds::Sounds::load(rate)?, rate)?;
        let input_resampler = (input_rate != 16_000)
            .then(|| resample::InputResampler::new(input_rate))
            .transpose()?;
        devices::start(&output_stream)?;
        devices::start(&input_stream)?;
        let models_dir = wake::models_dir();
        let (wake, wake_models) = if models::missing(&models_dir) {
            (None, Some(models::fetch_in_background(models_dir)))
        } else {
            (wake::load_optional(&models_dir), None)
        };
        Ok(Self {
            commands,
            outbox: Outbox::new(events),
            levels,
            output_pcm,
            input_pcm,
            mixer,
            capture: capture::Capture::new(wake),
            wake_models,
            input_resampler,
            _streams: [output_stream, input_stream],
        })
    }

    fn attach_downloaded_wake(&mut self) {
        let Some(models) = &self.wake_models else {
            return;
        };
        match models.try_recv() {
            Ok(dir) => {
                self.capture.set_wake(wake::load_optional(&dir));
                self.wake_models = None;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.wake_models = None,
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }

    fn run(mut self) {
        let mut output = [0.0; WORKER_CHUNK_SAMPLES];
        let mut input = [0; WORKER_CHUNK_SAMPLES];
        let mut levels_at = Instant::now();

        loop {
            if !self.apply_commands() || !self.outbox.flush() {
                return;
            }
            self.attach_downloaded_wake();

            let mut busy = false;
            loop {
                let count = pop_into(&mut self.input_pcm, &mut input);
                if count == 0 {
                    break;
                }
                busy = true;
                let events = match &mut self.input_resampler {
                    Some(resampler) => self.capture.push(&resampler.push(&input[..count])),
                    None => self.capture.push(&input[..count]),
                };
                self.outbox.push(events);
            }

            let output_count = self.output_pcm.slots().min(output.len());
            if output_count > 0 {
                busy = true;
                self.mixer.pull(&mut output[..output_count]);
                let _ = self.output_pcm.push_entire_slice(&output[..output_count]);
            }

            if levels_at.elapsed() >= LEVELS_EVERY {
                levels_at = Instant::now();
                self.publish_levels();
            }
            if !busy {
                thread::sleep(IDLE_SLEEP);
            }
        }
    }

    /// `false` khi hub-core đã đóng channel lệnh.
    fn apply_commands(&mut self) -> bool {
        loop {
            match self.commands.try_recv() {
                Ok(command) => {
                    let flush_preroll = matches!(&command, AudioCommand::Mic { on: true });
                    self.capture.apply(&command);
                    self.mixer.apply(command);
                    if flush_preroll {
                        self.outbox.push(self.capture.push(&[]));
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => return true,
                Err(mpsc::error::TryRecvError::Disconnected) => return false,
            }
        }
    }

    /// Chỉ báo UI khi mức đổi thật; lúc im lặng không đánh thức event loop.
    fn publish_levels(&self) {
        let next = Levels::from_rms(self.capture.level(), self.mixer.level());
        self.levels.send_if_modified(|shown| {
            let changed = next.differs_from(shown);
            if changed {
                *shown = next;
            }
            changed
        });
    }
}

fn pop_into(ring: &mut Consumer<i16>, buffer: &mut [i16]) -> usize {
    let mut count = 0;
    while count < buffer.len() {
        let Ok(sample) = ring.pop() else {
            break;
        };
        buffer[count] = sample;
        count += 1;
    }
    count
}

/// Gửi sự kiện cho hub-core mà không chặn thread audio: hub-core chậm thì
/// sự kiện xếp hàng ở đây thay vì làm TTS/chuông bị hụt mẫu.
struct Outbox {
    tx: mpsc::Sender<AudioEvent>,
    pending: VecDeque<AudioEvent>,
    dropped: bool,
}

impl Outbox {
    fn new(tx: mpsc::Sender<AudioEvent>) -> Self {
        Self {
            tx,
            pending: VecDeque::new(),
            dropped: false,
        }
    }

    fn push(&mut self, events: Vec<AudioEvent>) {
        self.pending.extend(events);
        if self.pending.len() > MAX_PENDING_EVENTS {
            self.pending
                .drain(..self.pending.len() - MAX_PENDING_EVENTS);
            if !self.dropped {
                eprintln!("hub-core không kịp nhận sự kiện audio; bỏ sự kiện cũ");
                self.dropped = true;
            }
        }
    }

    /// `false` khi hub-core đã đóng channel sự kiện.
    fn flush(&mut self) -> bool {
        while let Some(event) = self.pending.pop_front() {
            match self.tx.try_send(event) {
                Ok(()) => self.dropped = false,
                Err(mpsc::error::TrySendError::Full(event)) => {
                    self.pending.push_front(event);
                    return true;
                }
                Err(mpsc::error::TrySendError::Closed(_)) => return false,
            }
        }
        true
    }
}
