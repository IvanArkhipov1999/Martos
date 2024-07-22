use crate::ports::{Port, PortTrait};

/// Type for tick counting. It is signed for synchronization. It should be u128.
pub type TickType = u64;

/// The definition of the timers themselves.
/// TODO: Should contain synchronization period and synchronization scale.
pub struct Timer {
    /// Number of ticks in timer.
    tick_counter: TickType,
}

/// Operating system timer.
// TODO: Maybe it can be non static. It is static to make functions to pass to task manager.
// TODO: Default parameters should be read from config file.
static mut TIMER: Timer = Timer { tick_counter: 0 };

impl Timer {
    /// Setup function. May be used for setting configuration parameters.
    pub fn setup_timer() {
        Port::setup_hardware_timer()
    }

    /// Starts timer ticking.
    // TODO: What should happen after overflow?
    pub fn loop_timer() {
        unsafe {
            TIMER.tick_counter += 1;
        }
    }

    /// Stops timer ticking. By default, it does not stop.
    pub fn stop_condition_timer() -> bool {
        false
    }

    /// Returns tick counter.
    pub fn get_tick_counter() -> TickType {
        Port::get_tick_counter()
    }
}
