// TODO: maybe all this should be in martos, not in c-library folder

#![no_std]

use esp32_hal as _;
use esp32_hal::xtensa_lx_rt::exception::{Context, ExceptionCause};
use martos::task_manager::TaskManager;
use martos::timer::{TickType, Timer};

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_kernel_exception(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Kernel Exception: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_user_exception(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked User Exception: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_double_exception(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Double Exception: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_level_2_interrupt(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Level 2 Interrupt: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_level_3_interrupt(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Level 3 Interrupt: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_level_4_interrupt(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Level 4 Interrupt: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_level_5_interrupt(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Level 5 Interrupt: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_level_6_interrupt(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Level 6 Interrupt: {:?}, {:08x?}", cause, save_frame)
}

#[no_mangle]
#[link_section = ".rwtext"]
extern "C" fn __naked_level_7_interrupt(cause: ExceptionCause, save_frame: &Context) {
    panic!("Naked Level 7 Interrupt: {:?}, {:08x?}", cause, save_frame)
}

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
