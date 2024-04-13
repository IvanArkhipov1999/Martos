#![no_std]

mod ports;
pub mod task_manager;
pub mod timer;

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
use ports::xtensa_esp32::memory_manager::init_heap;

pub fn init_system() {
    #[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
    init_heap();
}
