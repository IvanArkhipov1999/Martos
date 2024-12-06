use esp_hal::timer::timg::{Timer, Timer0, Timer1, TimerGroup};
use esp_hal::{peripherals::*, prelude::*};

use crate::timer::TickType;

// TODO: initialize peripherals in separate mod
pub static mut TIMER00: Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>> = None;
pub static mut TIMER01: Option<Timer<Timer1<TIMG0>, esp_hal::Blocking>> = None;
pub static mut PERIFERALS_RNG: Option<RNG> = None;
pub static mut PERIFERALS_RADIO_CLK: Option<RADIO_CLK> = None;
pub static mut PERIFERALS_WIFI: Option<WIFI> = None;

/// Esp32 hardware timer setup.
pub fn setup_hardware_timer() {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);

    let timer00 = timer_group0.timer0;
    let timer01 = timer_group0.timer1;

    unsafe {
        TIMER00 = Some(timer00);
        TIMER01 = Some(timer01);
        PERIFERALS_RNG = Some(peripherals.RNG);
        PERIFERALS_RADIO_CLK = Some(peripherals.RADIO_CLK);
        PERIFERALS_WIFI = Some(peripherals.WIFI);
    }
}

/// Esp32 getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    esp_hal::time::now().ticks()
}
