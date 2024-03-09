// TODO: maybe all this should be in martos, not in c-library folder

#![no_std]

use ma_rtos::task_manager::TaskExecutor;
use ma_rtos::timer::{TickType, Timer};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
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
pub extern "C" fn new_task_executor() -> TaskExecutor {
    TaskExecutor::new()
}
