pub(crate) mod calendar;
pub(crate) mod countdowns;
mod dispatch;
pub(crate) mod home;
pub(crate) mod photos;
pub(crate) mod registry;
pub(crate) mod reminders;
pub(crate) mod timers;
pub(crate) mod tool_args;
pub(crate) mod weather;

pub(crate) use dispatch::{Cx, Feature};
