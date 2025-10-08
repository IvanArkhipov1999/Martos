#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use esp_backtrace as _;
use esp_hal::uart::config::{Config, DataBits, StopBits};
use esp_hal::Blocking;
use esp_hal::{entry, uart::Uart};
use esp_println::println;
use martos::{
    get_io, get_uart2, init_system,
    task_manager::{TaskManager, TaskManagerTrait},
};

/// Counter to track processed bytes
static BYTE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// UART instance (initialized in setup)
static mut UART_INSTANCE: Option<Uart<'static, esp_hal::peripherals::UART0, Blocking>> = None;

/// Setup function for task to execute.
fn setup_fn() {
    println!("UART Echo Setup started");

    unsafe {
        // Get UART0 and IO from Martos (ESP32-C6 uses UART0)
        let uart0 = get_uart2();
        let io = get_io();

        // UART configuration: 19200 baud, 8N1
        let config = Config::default()
            .baudrate(19200)
            .data_bits(DataBits::DataBits8)
            .parity_none()
            .stop_bits(StopBits::STOP1);

        // Initialize UART
        let uart = Uart::new_with_config(
            uart0,
            config,
            io.pins.gpio16, // RX pin
            io.pins.gpio17, // TX pin
        )
        .expect("UART init failed");

        UART_INSTANCE = Some(uart);

        println!("UART Echo ready on GPIO16(RX)/GPIO17(TX) at 19200 baud");
    }
}

/// Loop function for task to execute.
fn loop_fn() {
    unsafe {
        if let Some(ref mut uart) = UART_INSTANCE {
            let mut buffer = [0u8; 1];

            // Try to read a byte using read_bytes method
            if uart.read_bytes(&mut buffer).is_ok() {
                let byte = buffer[0];
                let count = BYTE_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

                println!(
                    "Received byte #{}: 0x{:02X} ('{}') - echoing back",
                    count,
                    byte,
                    if byte.is_ascii_graphic() || byte == b' ' {
                        byte as char
                    } else {
                        '.'
                    }
                );

                // Echo the byte back using write_bytes method
                if uart.write_bytes(&buffer).is_err() {
                    println!("Failed to echo byte!");
                }
            }
        }
    }
}

/// Stop condition function for task to execute.
fn stop_condition_fn() -> bool {
    // Never stop - run forever
    false
}

#[entry]
fn main() -> ! {
    // Initialize Martos (including UART)
    init_system();

    // Add task to execute
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);

    // Start task manager
    TaskManager::start_task_manager();
}
