//! UART module for ESP32 - handles UART and GPIO pin peripherals

use super::peripherals::init_peripherals;
use super::peripherals::{UartPeriph, PERIFERALS_GPIO16, PERIFERALS_GPIO17, PERIFERALS_UART};
use core::sync::atomic::{AtomicBool, Ordering};
use esp_hal::peripherals::{GPIO16, GPIO17};

// Static storage for UART RX/TX pins
pub static mut UART_PINS: Option<IoType> = None;

// Flag to ensure UART is initialized only once
static UART_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// ESP32 UART type alias (depends on arch)
pub type Uart2Type = UartPeriph;

/// ESP32 UART pin pair type alias (RX, TX)
pub type IoType = (GPIO16<'static>, GPIO17<'static>);

/// Initialize UART subsystem for ESP32
///
/// This function initializes the UART pins separately from the hardware
/// timer initialization. It calls init_peripherals to get fresh peripheral
/// instances.
pub fn setup_uart() {
    if UART_INITIALIZED
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        init_peripherals();
        unsafe {
            let gpio16 = PERIFERALS_GPIO16.take().expect("GPIO16 peripherals error");
            let gpio17 = PERIFERALS_GPIO17.take().expect("GPIO17 peripherals error");
            UART_PINS = Some((gpio16, gpio17));
        }
    }
}

/// Get UART peripheral instance
///
/// Returns the UART peripheral for configuration. Can only be called once.
pub fn get_uart2() -> UartPeriph {
    unsafe {
        PERIFERALS_UART
            .take()
            .expect("UART2 not available - call setup_uart first")
    }
}

/// Get UART RX/TX pin instances
///
/// Returns the (RX, TX) pin pair for UART configuration. Can only be called once.
pub fn get_io() -> IoType {
    unsafe {
        UART_PINS
            .take()
            .expect("UART pins not available - call setup_uart first")
    }
}

/// Check if UART is initialized
pub fn is_uart_initialized() -> bool {
    UART_INITIALIZED.load(Ordering::Acquire)
}
