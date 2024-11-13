#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use esp_println::println;
use martos::{init_system, task_manager::TaskManager};
use martos::task_manager::TaskManagerTrait;
use esp_backtrace as _;
use esp_hal::xtensa_lx_rt::xtensa_lx::timer::delay;

/// Counter to work with in loop.
static COUNTER: AtomicU32 = AtomicU32::new(1);

/// Loop function for task to execute.
fn loop_fn_1() {
    loop {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        println!("Loop 0; Counter = {}", unsafe { COUNTER.as_ptr().read() });
        delay(50_000_000);
    }
}

fn loop_fn_2() {
    loop {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        println!("Loop 1; Counter = {}", unsafe { COUNTER.as_ptr().read() });
        delay(50_000_000);
    }
}

fn temp(){}
fn temp_2()-> bool{true}

#[entry]
fn main() -> ! {
    // Initialize Martos.
    init_system();
    // Add task to execute.
    TaskManager::add_task(temp, loop_fn_1, temp_2);
    TaskManager::add_task(temp, loop_fn_2, temp_2);
    // Start task manager.
    TaskManager::start_task_manager();
}