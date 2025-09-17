#![no_std]
extern crate alloc;

mod ports;
use ports::PortTrait;
#[cfg(feature = "c-library")]
pub mod c_api;
pub mod task_manager;
pub mod timer;
#[cfg(feature = "network")]
pub mod time_sync;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
#[cfg(feature = "network")]
use esp_wifi::esp_now::EspNow;

/// Martos initialization. Should be called before using Martos functions.
pub fn init_system() {
    // Memory initialization.
    ports::Port::init_heap();
    // Hardware timer setup.
    ports::Port::setup_hardware_timer();
    #[cfg(feature = "network")]
    // Network setup.
    ports::Port::init_network();
}

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
#[cfg(feature = "network")]
pub fn get_esp_now() -> EspNow<'static> {
    return ports::Port::get_esp_now();
}
