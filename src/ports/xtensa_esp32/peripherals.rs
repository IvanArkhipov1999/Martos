use core::sync::atomic::{AtomicBool, Ordering};
use esp_hal::peripherals::*;

// Single-shot initialization guard for peripherals
static PERIPHERALS_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Raw peripheral handles stored for later split/consumption by subsystems
pub static mut PERIFERALS_TIMG0: Option<TIMG0> = None;
pub static mut PERIFERALS_TIMG1: Option<TIMG1> = None;
pub static mut PERIFERALS_WIFI: Option<WIFI> = None;
// Software interrupt used by esp-rtos for context switching
pub static mut PERIFERALS_FROM_CPU_INTR0: Option<FROM_CPU_INTR0> = None;
// Select UART peripheral depending on the target architecture
#[cfg(target_arch = "xtensa")]
pub type UartPeriph = UART2<'static>;
#[cfg(target_arch = "riscv32")]
pub type UartPeriph = UART0<'static>;

pub static mut PERIFERALS_UART: Option<UartPeriph> = None;
// GPIO pins used for UART RX/TX
pub static mut PERIFERALS_GPIO16: Option<GPIO16> = None;
pub static mut PERIFERALS_GPIO17: Option<GPIO17> = None;

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
            PERIFERALS_WIFI = Some(peripherals.WIFI);
            PERIFERALS_FROM_CPU_INTR0 = Some(peripherals.FROM_CPU_INTR0);
            #[cfg(target_arch = "xtensa")]
            {
                PERIFERALS_UART = Some(peripherals.UART2);
            }
            #[cfg(target_arch = "riscv32")]
            {
                PERIFERALS_UART = Some(peripherals.UART0);
            }
            PERIFERALS_GPIO16 = Some(peripherals.GPIO16);
            PERIFERALS_GPIO17 = Some(peripherals.GPIO17);
        }
    }
}

/// Check if peripherals were initialized
pub fn is_initialized() -> bool {
    PERIPHERALS_INITIALIZED.load(Ordering::Acquire)
}
