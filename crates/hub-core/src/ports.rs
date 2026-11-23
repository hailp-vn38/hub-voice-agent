use jiff::Zoned;

/// Source of the current time. Time always enters hub-core through this port.
pub trait Clock: Send + Sync {
    fn now(&self) -> Zoned;
}

/// Wall clock in the system time zone.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Zoned {
        Zoned::now()
    }
}

/// Reports whether wall time is reliable.
pub trait System: Send + Sync {
    fn time_synced(&self) -> bool;
}
