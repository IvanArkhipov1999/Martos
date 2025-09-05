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

#[cfg(target_arch = "xtensa")]
use crate::ports::xtensa_esp32::XtensaEsp32Port as CurrentPort;

#[cfg(target_arch = "mips")]
use crate::ports::mips64::Mips64 as CurrentPort;

#[cfg(not(any(target_arch = "xtensa", target_arch = "mips")))]
use crate::ports::mok::Mok as CurrentPort;

/// Martos initialization. Should be called before using Martos functions.
pub fn init_system() {
    // Memory initialization.
    CurrentPort::init_heap();
    // Hardware timer setup.
    CurrentPort::setup_hardware_timer();
    #[cfg(feature = "network")]
    // Network setup.
    CurrentPort::init_network();
    // Uart setup.
    #[cfg(feature = "uart")]
    CurrentPort::setup_uart();
}

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
#[cfg(feature = "network")]
pub fn get_esp_now() -> EspNow<'static> {
    return CurrentPort::get_esp_now();
}

#[cfg(feature = "uart")]
pub fn get_uart2() -> <CurrentPort as crate::ports::PortTrait>::Uart2Type {
    CurrentPort::get_uart2()
}

#[cfg(feature = "uart")]
pub fn get_io() -> <CurrentPort as crate::ports::PortTrait>::IoType {
    CurrentPort::get_io()
}
