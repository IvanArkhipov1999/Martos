#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
use crate::ports::mok::hardware_timer::{get_tick_counter, setup_hardware_timer};
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
use crate::ports::xtensa_esp32::hardware_timer::{get_tick_counter, setup_hardware_timer};

/// Type for tick counting. It is signed for synchronization. It should be u128.
pub type TickType = i64;

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
        setup_hardware_timer()
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
        get_tick_counter()
    }
}
