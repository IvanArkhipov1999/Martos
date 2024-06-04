#![no_std]

mod ports;
pub mod task_manager;
pub mod timer;

#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
use ports::mok::hardware_timer::setup_hardware_timer;
#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
use ports::mok::memory_manager::init_heap;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
use ports::xtensa_esp32::hardware_timer::setup_hardware_timer;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
use ports::xtensa_esp32::memory_manager::init_heap;

/// Martos initialization. Should be called before using Martos functions.
pub fn init_system() {
    // Memory initialization.
    init_heap();
    // Hardware timer setup.
    setup_hardware_timer();
}
