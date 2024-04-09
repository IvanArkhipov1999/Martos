#![no_std]
#![no_main]

// TODO: move this to ports of Martos with conditions
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use esp32_hal::entry;
use esp_backtrace as _;
use esp_println::println;
use martos::task_manager::TaskManager;

/// Counter to work with in loop.
static COUNTER: AtomicU32 = AtomicU32::new(1);

/// Setup function for task to execute.
fn setup_fn() {
    println!("Setup hello world!")
}

/// Loop function for task to execute.
fn loop_fn() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
    println!("Loop hello world!");
    println!("Counter = {}", unsafe { COUNTER.as_ptr().read() });
}

/// Stop condition function for task to execute.
fn stop_condition_fn() -> bool {
    let value = unsafe { COUNTER.as_ptr().read() };
    if value % 50 == 0 {
        return true;
    }
    return false;
}

#[entry]
fn main() -> ! {
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);

    // Add task to execute.
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);
    // Start task manager.
    TaskManager::start_task_manager();
}
