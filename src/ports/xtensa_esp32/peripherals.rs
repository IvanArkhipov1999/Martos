use core::sync::atomic::{AtomicBool, Ordering};
use esp_hal::peripherals::*;

// Single-shot initialization guard for peripherals
static PERIPHERALS_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Raw peripheral handles stored for later split/consumption by subsystems
pub static mut PERIFERALS_TIMG0: Option<TIMG0> = None;
pub static mut PERIFERALS_TIMG1: Option<TIMG1> = None;
pub static mut PERIFERALS_RNG: Option<RNG> = None;
pub static mut PERIFERALS_RADIO_CLK: Option<RADIO_CLK> = None;
pub static mut PERIFERALS_WIFI: Option<WIFI> = None;
pub static mut PERIFERALS_UART2: Option<UART2> = None;
pub static mut PERIFERALS_GPIO: Option<GPIO> = None;
pub static mut PERIFERALS_IO_MUX: Option<IO_MUX> = None;

/// Initialize ESP32 peripherals once and store raw handles for later use
pub fn init_peripherals() {
    if PERIPHERALS_INITIALIZED
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        let peripherals = esp_hal::init(esp_hal::Config::default());

        unsafe {
            PERIFERALS_TIMG0 = Some(peripherals.TIMG0);
            PERIFERALS_TIMG1 = Some(peripherals.TIMG1);
            PERIFERALS_RNG = Some(peripherals.RNG);
            PERIFERALS_RADIO_CLK = Some(peripherals.RADIO_CLK);
            PERIFERALS_WIFI = Some(peripherals.WIFI);
            PERIFERALS_UART2 = Some(peripherals.UART2);
            PERIFERALS_GPIO = Some(peripherals.GPIO);
            PERIFERALS_IO_MUX = Some(peripherals.IO_MUX);
        }
    }
}

/// Check if peripherals were initialized
pub fn is_initialized() -> bool {
    PERIPHERALS_INITIALIZED.load(Ordering::Acquire)
}
