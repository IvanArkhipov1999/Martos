//! UART module for ESP32 - handles UART2 and IO peripherals

use core::sync::atomic::{AtomicBool, Ordering};
use esp_hal::{gpio::*, peripherals::*};

// Static variables for UART peripherals
pub static mut PERIFERALS_UART2: Option<UART2> = None;
pub static mut IO: Option<Io> = None;

// Flag to ensure UART is initialized only once
static UART_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// ESP32 UART2 type alias
pub type Uart2Type = UART2;

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
        // Initialize ESP-HAL to get peripheral instances
        let peripherals = esp_hal::init(esp_hal::Config::default());
        
        unsafe {
            PERIFERALS_UART2 = Some(peripherals.UART2);
            let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
            IO = Some(io);
        }
    }
}

/// Get UART2 peripheral instance
/// 
/// Returns the UART2 peripheral for configuration. Can only be called once.
pub fn get_uart2() -> UART2 {
    unsafe {
        PERIFERALS_UART2.take().expect("UART2 not available - call setup_uart first")
    }
}

/// Get IO peripheral instance
/// 
/// Returns the IO peripheral for pin configuration. Can only be called once.
pub fn get_io() -> Io {
    unsafe {
        IO.take().expect("IO not available - call setup_uart first")
    }
}

/// Check if UART is initialized
pub fn is_uart_initialized() -> bool {
    UART_INITIALIZED.load(Ordering::Acquire)
}
