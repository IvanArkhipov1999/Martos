use core::time::Duration;

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

    /// Starts the hardware timer.
    pub fn start_timer() {
        Port::start_hardware_timer()
    }

    /// Changes the timer period.
    pub fn change_period_timer(period: Duration) {
        Port::change_period_timer(period);
    }

    /// Starts timer ticking.
    // TODO: What should happen after overflow?
    pub fn loop_timer() {
        unsafe {
            TIMER.tick_counter += 1;
        }
    }

    /// Stops timer ticking. Returns false if the device doesn't support stopping the counter.
    pub fn stop_condition_timer() -> bool {
        Port::stop_hardware_timer()
    }

    /// Returns current counter value.
    pub fn get_time() -> Duration {
        Port::get_time()
    }
}
