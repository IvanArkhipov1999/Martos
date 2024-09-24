#![no_std]

use esp_hal as _;
use martos::c_api::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

