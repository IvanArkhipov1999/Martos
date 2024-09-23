// TODO: maybe all this should be in martos, not in c-library folder

#![no_std]

use martos::c_api::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
