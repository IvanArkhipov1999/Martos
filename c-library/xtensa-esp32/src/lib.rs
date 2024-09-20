// TODO: maybe all this should be in martos, not in c-library folder

#![no_std]

use esp_hal as _;
use martos::task_manager::TaskManager;
use martos::timer::{TickType, Timer};
use martos::src::c_api::init_system;
use martos::src::c_api::setup_timer;
use martos::src::c_api::loop_timer;
use martos::src::c_api::stop_condition_timer;
use martos::src::c_api::get_tick_counter;
use martos::src::c_api::add_task;
use martos::src::c_api::start_task_manager;
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
