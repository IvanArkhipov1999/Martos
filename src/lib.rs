#![no_std]

mod ports;
use ports::Port;
pub mod task_manager;
pub mod timer;


/// Martos initialization. Should be called before using Martos functions.
pub fn init_system() {
    // Memory initialization.
    ports::PORT::init_heap();
    // Hardware timer setup.
    ports::PORT::setup_hardware_timer();
}
