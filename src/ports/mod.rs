use crate::timer::TickType;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
use esp_wifi::esp_now::EspNow;

/// PortTrait contains all the platform specific functions.
pub trait PortTrait {
    /// Function is called when timer is created. Can be used to set configuration.
    fn setup_hardware_timer();
    /// Function used to get amount of ticks from the start of a timer
    fn get_tick_counter() -> TickType;
    /// Function is called when heap is created. Can be used to set configuration.
    fn init_heap();
    /// Function for initializing network settings.
    fn init_network();
    #[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
    fn get_esp_now() -> EspNow<'static>;
}

/// Port is an alias of PortTrait implementation for a current platform

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub mod xtensa_esp32;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub type Port = xtensa_esp32::XtensaEsp32;

#[cfg(all(
    not(any(target_arch = "riscv32", target_arch = "xtensa")),
    not(target_arch = "mips64")
))]
pub mod mok;
#[cfg(all(
    not(any(target_arch = "riscv32", target_arch = "xtensa")),
    not(target_arch = "mips64")
))]
pub type Port = mok::Mok;

#[cfg(target_arch = "mips64")]
pub mod mips64;
#[cfg(target_arch = "mips64")]
pub type Port = mips64::Mips64;
