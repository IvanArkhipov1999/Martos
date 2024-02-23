use crate::timer::Timer;

use core::sync::atomic::{AtomicU32, Ordering};

mod task_manager;
mod timer;

static COUNTER: AtomicU32 = AtomicU32::new(1);
static COUNTER2: AtomicU32 = AtomicU32::new(1);

fn setup_fn() {
    println!("Setup!")
}
fn loop_fn() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
    println!("Counter {}", unsafe { COUNTER.as_ptr().read() });
    println!("Loop!")
}

fn stop_condition_fn() -> bool {
    let value = unsafe { COUNTER.as_ptr().read() };
    if value % 50 == 0 {
        return true;
    }
    return false;
}

fn setup_fn2() {
    println!("Setup2!")
}
fn loop_fn2() {
    COUNTER2.fetch_add(1, Ordering::Relaxed);
    println!("Counter2 {}", unsafe { COUNTER2.as_ptr().read() });
    println!("Loop2!")
}

fn stop_condition_fn2() -> bool {
    let value = unsafe { COUNTER2.as_ptr().read() };
    if value % 55 == 0 {
        return true;
    }
    return false;
}

fn setup_time() {}
fn loop_time() {
    println!("{}", Timer::get_tick_counter())
}
fn stop_condition_time() -> bool {
    if Timer::get_tick_counter() > 100 {
        return true;
    }
    return false;
}

fn main() {
    let mut task_executor = task_manager::TaskExecutor::new();
    task_executor.add_task(setup_fn, loop_fn, stop_condition_fn);
    task_executor.add_task(setup_fn2, loop_fn2, stop_condition_fn2);

    // Timer execution
    task_executor.add_task(
        Timer::setup_timer,
        Timer::loop_timer,
        Timer::stop_condition_timer,
    );
    task_executor.add_task(setup_time, loop_time, stop_condition_time);

    task_executor.start_task_manager();
}
