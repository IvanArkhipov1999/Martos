#![no_std]
#![no_main]

#[macro_use]
extern crate cortex_m_rt;
use martos::init_system;
use martos::task_manager::TaskManager;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Setup function for task to execute.
fn setup_fn() {}

/// Loop function for task to execute.
fn loop_fn() {}

/// Stop condition function for task to execute.
fn stop_condition_fn() -> bool {
    return false;
}

#[entry]
fn main() -> ! {
    // Initialize Martos.
    init_system();
    // Add task to execute.
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);
    // Start task manager.
    TaskManager::start_task_manager();
}
