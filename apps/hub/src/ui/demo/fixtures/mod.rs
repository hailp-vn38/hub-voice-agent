//! Fixture và kịch bản demo, mỗi tính năng một module.
pub(super) mod alerts;
pub(super) mod calendar;
pub(super) mod center;
pub(super) mod countdowns;
pub(super) mod first_boot;
pub(super) mod home;
pub(super) mod menu;
pub(super) mod photos;
pub(super) mod reminders;
pub(super) mod response;
pub(super) mod settings;
pub(super) mod spawn;
pub(super) mod timers;
pub(super) mod weather;
mod wiring;

pub(super) use wiring::{init, tick};
