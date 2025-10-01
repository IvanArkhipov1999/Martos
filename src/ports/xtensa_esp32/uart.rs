//! UART module for ESP32 - handles UART2 and IO peripherals

use super::peripherals::init_peripherals;
use super::peripherals::{UartPeriph, PERIFERALS_GPIO, PERIFERALS_IO_MUX, PERIFERALS_UART};
use core::sync::atomic::{AtomicBool, Ordering};
use esp_hal::{gpio::*, peripherals::*};

// Static variables for UART peripherals
pub static mut IO: Option<Io> = None;

// Flag to ensure UART is initialized only once
static UART_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// ESP32 UART type alias (depends on arch)
pub type Uart2Type = UartPeriph;

/// ESP32 IO type alias  
pub type IoType = Io;

/// Initialize UART subsystem for ESP32
///
/// This function initializes the UART2 and IO peripherals separately from
/// the hardware timer initialization. It calls esp_hal::init to get fresh
/// peripheral instances.
pub fn setup_uart() {
    if UART_INITIALIZED
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        init_peripherals();
        unsafe {
            let gpio = PERIFERALS_GPIO.take().expect("GPIO peripherals error");
            let io_mux = PERIFERALS_IO_MUX.take().expect("IO_MUX peripherals error");
            let io = Io::new(gpio, io_mux);
            IO = Some(io);
        }
    }
}

/// Get UART2 peripheral instance
///
/// Returns the UART2 peripheral for configuration. Can only be called once.
pub fn get_uart2() -> UartPeriph {
    unsafe {
        PERIFERALS_UART
            .take()
            .expect("UART2 not available - call setup_uart first")
    }
}

/// Get IO peripheral instance
///
/// Returns the IO peripheral for pin configuration. Can only be called once.
pub fn get_io() -> Io {
    unsafe { IO.take().expect("IO not available - call setup_uart first") }
}

/// Check if UART is initialized
pub fn is_uart_initialized() -> bool {
    UART_INITIALIZED.load(Ordering::Acquire)
}
