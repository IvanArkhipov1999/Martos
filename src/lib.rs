#![no_std]

mod ports;
use ports::PortTrait;
#[cfg(feature = "c-library")]
pub mod c_api;
pub mod task_manager;
pub mod timer;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
use esp_wifi::esp_now::EspNow;

/// Martos initialization. Should be called before using Martos functions.
pub fn init_system() {
    // Memory initialization.
    ports::Port::init_heap();
    // Hardware timer setup.
    ports::Port::setup_hardware_timer();
    // Network setup.
    ports::Port::init_network();
}

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub fn get_esp_now() -> EspNow<'static> {
    return ports::Port::get_esp_now();
}
