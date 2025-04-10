#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use martos::{
    init_system,
    task_manager::{TaskManager, TaskManagerTrait},
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static COUNTER: AtomicU32 = AtomicU32::new(1);

/// Setup function for the main task.
fn main_task_setup_fn() {}

/// Loop function for the main task.
fn main_task_loop_fn() {
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    if count == 25 {
        TaskManager::add_task(
            inner_task_setup_fn,
            inner_task_loop_fn,
            inner_task_stop_condition_fn,
        );
    }
}

/// Stop condition for the main task.
fn main_task_stop_condition_fn() -> bool {
    let count = unsafe { COUNTER.as_ptr().read() };
    count == 25
}

/// Setup function for the inner task.
fn inner_task_setup_fn() {}

/// Loop function for the inner task.
fn inner_task_loop_fn() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Stop condition for the inner task.
fn inner_task_stop_condition_fn() -> bool {
    let count = unsafe { COUNTER.as_ptr().read() };
    count % 10 == 0
}

#[no_mangle]
pub extern "C" fn __start() -> ! {
    // Initialize Martos.
    init_system();
    // Add task to execute.
    TaskManager::add_task(
        main_task_setup_fn,
        main_task_loop_fn,
        main_task_stop_condition_fn,
    );
    // Start task manager.
    TaskManager::start_task_manager();
}
