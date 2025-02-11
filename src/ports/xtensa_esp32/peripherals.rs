use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use esp_hal::timer::timg::{Timer, Timer0, TimerGroup};
use esp_hal::{peripherals::*, prelude::*};

pub static mut TIMER00: Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>> = None;
pub static mut TIMER10: Option<Timer<Timer0<TIMG1>, esp_hal::Blocking>> = None;
pub static mut PERIFERALS_RNG: Option<RNG> = None;
pub static mut PERIFERALS_RADIO_CLK: Option<RADIO_CLK> = None;
pub static mut PERIFERALS_WIFI: Option<WIFI> = None;

pub fn init_peripherals() {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);

    let timer00 = timer_group0.timer0;
    let timer10 = timer_group1.timer0;

    unsafe {
        TIMER00 = Some(timer00);
        TIMER10 = Some(timer10);
        PERIFERALS_RNG = Some(peripherals.RNG);
        PERIFERALS_RADIO_CLK = Some(peripherals.RADIO_CLK);
        PERIFERALS_WIFI = Some(peripherals.WIFI);
    }
}
