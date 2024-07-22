use crate::timer::TickType;

/// PortTrait contains all the platform specific functions.
pub trait PortTrait {
    /// Function is called when timer is created. Can be used to set configuration.
    fn setup_hardware_timer();
    /// Function used to get amount of ticks from the start of a timer
    fn get_tick_counter() -> TickType;

    /// Function is called when heap is created. Can be used to set configuration.
    fn init_heap();
}

/// Port is an alias of PortTrait implementation for a current platform

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub mod xtensa_esp32;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub type Port = xtensa_esp32::XtensaEsp32;

#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
pub mod mok;
#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
pub type Port = mok::Mok;
