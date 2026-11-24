mod capture;
mod devices;
mod mixer;
mod resample;
mod sounds;
mod wake;
mod worker;

use hub_core::{AudioCommand, AudioEvent};
use tokio::sync::{mpsc, watch};

pub use worker::spawn;

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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
