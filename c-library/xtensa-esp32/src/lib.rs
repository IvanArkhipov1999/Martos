use ma_rtos::timer::{Timer, TickType};

#[no_mangle]
pub extern "C" fn get_tick_counter() -> TickType {
    Timer::get_tick_counter()
}