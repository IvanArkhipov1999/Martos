#![no_std]
mod ports;
use ports::PortTrait;
pub mod task_manager;
pub mod timer;
pub mod c_api;

/// Martos initialization. Should be called before using Martos functions.
pub fn init_system() {
    // Memory initialization.
    ports::Port::init_heap();
    // Hardware timer setup.
    ports::Port::setup_hardware_timer();
}
