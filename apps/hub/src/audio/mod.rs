mod capture;
mod devices;
mod levels;
mod mixer;
mod resample;
mod sounds;
mod wake;
mod worker;

use hub_core::{AudioCommand, AudioEvent};
use tokio::sync::{mpsc, watch};

pub use levels::Levels;
pub use worker::{disabled, spawn};

pub struct AudioHandle {
    pub commands: mpsc::Sender<AudioCommand>,
    pub events: mpsc::Receiver<AudioEvent>,
    pub levels: watch::Receiver<Levels>,
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
