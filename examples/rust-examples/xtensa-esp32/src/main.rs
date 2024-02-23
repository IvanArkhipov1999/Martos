#![no_std]
#![no_main]

use esp32_hal::entry;
use esp_backtrace as _;
use esp_println::println;
use ma_rtos::timer::Timer;

#[entry]
fn main() -> ! {
    println!("Timer {}", Timer::get_tick_counter());
    loop {}
}