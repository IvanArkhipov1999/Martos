use crate::timer::TickType;

/// PortTrait contains all the platform specific functions.
pub trait PortTrait {
    fn setup_hardware_timer();
    fn get_tick_counter() -> TickType;

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
