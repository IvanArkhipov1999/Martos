// TODO: maybe all this should be in martos, not in c-library folder

#![no_std]

use esp_hal as _;
use martos::task_manager::TaskManager;
use martos::timer::{TickType, Timer};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn init_system() {
    martos::init_system()
}

#[no_mangle]
pub extern "C" fn setup_timer() {
    Timer::setup_timer()
}

#[no_mangle]
pub extern "C" fn loop_timer() {
    Timer::loop_timer()
}

#[no_mangle]
pub extern "C" fn stop_condition_timer() -> bool {
    Timer::stop_condition_timer()
}

#[no_mangle]
pub extern "C" fn get_tick_counter() -> TickType {
    Timer::get_tick_counter()
}

#[no_mangle]
pub extern "C" fn add_task(
    setup_fn: extern "C" fn() -> (),
    loop_fn: extern "C" fn() -> (),
    stop_condition_fn: extern "C" fn() -> bool,
) {
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn)
}

#[no_mangle]
pub extern "C" fn start_task_manager() {
    TaskManager::start_task_manager()
}
