use crate::timer::TickType;

/// Mips64 hardware timer setup.
pub fn setup_hardware_timer() {}

/// Mips64 getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    0
}
