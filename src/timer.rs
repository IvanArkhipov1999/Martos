use core::time::Duration;

use crate::ports::{Port, PortTrait};

/// Type for tick counting. It is signed for synchronization. It should be u128.
pub type TickType = u64;

/// The definition of the timers themselves.
/// TODO: Should contain synchronization period and synchronization scale.
pub struct Timer {
    /// Timer number in the timer block.
    timer_index: u8,
}

impl Timer {
    /// Setup function. May be used for setting configuration parameters.
    pub fn setup_timer() {
        Port::setup_hardware_timer()
    }

    /// Gets the timer instance at the specified index.
    pub fn get_timer(timer_index: u8) -> Option<Self> {
        if Port::valid_timer_index(timer_index) {
            Some(Self { timer_index })
        } else {
            None
        }
    }

    /// Starts the hardware timer.
    /// TODO: What should happen after overflow?
    pub fn start_timer(&self) {
        Port::start_hardware_timer(self.timer_index)
    }

    /// Updates the operating mode of the timer to be either an auto reload timer or a one-shot timer.
    pub fn set_reload_mode(&self, auto_reload: bool) {
        Port::set_reload_mode(self.timer_index, auto_reload)
    }

    /// Changes the timer period.
    pub fn change_period_timer(&self, period: Duration) {
        Port::change_period_timer(self.timer_index, period);
    }

    /// Stops timer ticking. Returns false if the device doesn't support stopping the counter.
    pub fn stop_condition_timer(&self) -> bool {
        Port::stop_hardware_timer(self.timer_index)
    }

    /// Returns current counter value.
    pub fn get_time(&self) -> Duration {
        Port::get_time(self.timer_index)
    }
}
