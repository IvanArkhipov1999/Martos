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
// Select UART peripheral depending on the target architecture
#[cfg(target_arch = "xtensa")]
pub type UartPeriph = UART2;
#[cfg(target_arch = "riscv32")]
pub type UartPeriph = UART0;

pub static mut PERIFERALS_UART: Option<UartPeriph> = None;
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
            #[cfg(target_arch = "xtensa")]
            {
                PERIFERALS_UART = Some(peripherals.UART2);
            }
            #[cfg(target_arch = "riscv32")]
            {
                PERIFERALS_UART = Some(peripherals.UART0);
            }
            PERIFERALS_GPIO = Some(peripherals.GPIO);
            PERIFERALS_IO_MUX = Some(peripherals.IO_MUX);
        }
    }
}

/// Check if peripherals were initialized
pub fn is_initialized() -> bool {
    PERIPHERALS_INITIALIZED.load(Ordering::Acquire)
}
