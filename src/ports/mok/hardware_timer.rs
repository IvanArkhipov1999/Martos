use crate::timer::TickType;
use core::time::Duration;

/// Mok hardware timer setup.
pub fn setup_hardware_timer() {}

/// Mok start harware timer.
pub fn start_hardware_timer() {}

/// Mok change the period of a timer.
pub fn change_period_timer(period: Duration) {}

/// Mok getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    0
}
