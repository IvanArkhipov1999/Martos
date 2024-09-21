use crate::{task_manager, timer};
use task_manager::TaskManager;
use timer::{TickType, Timer};

#[no_mangle]
pub extern "C" fn init_system() {
    super::init_system();
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
