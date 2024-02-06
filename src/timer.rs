/// Type for tick counting.
type TickType = u32;

/// The definition of the timers themselves.
#[derive(Debug)]
pub struct Timer {
    /// Name of timer. It may be useful for debugging and users.
    name: String,
    /// Number of ticks in timer by now
    now_time: TickType,
    /// Number of ticks, after what timer will be expired.
    expired_time: TickType,
}

impl Timer {
    /// Creating new timer.
    pub fn new(name: String, now_time: TickType, expired_time: TickType) -> Timer {
        Timer { name, now_time, expired_time }
    }
}
