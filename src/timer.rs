/// Type for tick counting.
type TickType = u32;

/// The definition of the timers themselves.
pub struct Timer {
    /// Name of timer. It may be useful for debugging and users.
    pub name: String,
    /// Number of ticks, after what timer will be expired.
    pub expired_time: TickType,
}
