#![no_std]
extern crate alloc;

mod ports;
use ports::PortTrait;
#[cfg(feature = "c-library")]
pub mod c_api;
pub mod task_manager;
pub mod timer;
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
    // Uart setup.
    #[cfg(feature = "uart")]
    ports::Port::setup_uart();
}

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
#[cfg(feature = "network")]
pub fn get_esp_now() -> EspNow<'static> {
    return ports::Port::get_esp_now();
}

#[cfg(feature = "uart")]
pub fn get_uart2() -> <CurrentPort as crate::ports::PortTrait>::Uart2Type {
    ports::Port::get_uart2()
}

#[cfg(feature = "uart")]
pub fn get_io() -> <CurrentPort as crate::ports::PortTrait>::IoType {
    ports::Port::get_io()
}
