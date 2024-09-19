// TODO: maybe all this should be in martos, not in c-library folder

#![no_std]

use esp_hal as _;
use martos::task_manager::TaskManager;
use martos::timer::{TickType, Timer};
use martos::c-api::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
