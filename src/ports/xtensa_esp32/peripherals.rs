use esp_hal::{peripherals::*, prelude::*};

pub static mut PERIFERALS_RNG: Option<RNG> = None;
pub static mut PERIFERALS_RADIO_CLK: Option<RADIO_CLK> = None;
pub static mut PERIFERALS_WIFI: Option<WIFI> = None;

pub fn init_peripherals() {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    unsafe {
        PERIFERALS_RNG = Some(peripherals.RNG);
        PERIFERALS_RADIO_CLK = Some(peripherals.RADIO_CLK);
        PERIFERALS_WIFI = Some(peripherals.WIFI);
    }
}
