#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use esp_backtrace as _;
use esp_hal::main;
use esp_hal::uart::{Config, DataBits, Parity, StopBits, Uart};
use esp_hal::Blocking;
use esp_println::println;
use martos::{
    get_io, get_uart2, init_system,
    task_manager::{TaskManager, TaskManagerTrait},
};

/// Counter to track processed bytes
static BYTE_COUNTER: AtomicU32 = AtomicU32::new(0);

/// UART instance (initialized in setup)
static mut UART_INSTANCE: Option<Uart<'static, Blocking>> = None;

/// Setup function for task to execute.
fn setup_fn() {
    println!("UART Echo Setup started");

    unsafe {
        // Get UART2 and RX/TX pins from Martos
        let uart2 = get_uart2();
        let (rx_pin, tx_pin) = get_io();

        // UART configuration: 19200 baud, 8N1
        let config = Config::default()
            .with_baudrate(19200)
            .with_data_bits(DataBits::_8)
            .with_parity(Parity::None)
            .with_stop_bits(StopBits::_1);

        // Initialize UART
        let uart = Uart::new(uart2, config)
            .expect("UART init failed")
            .with_rx(rx_pin) // RX pin
            .with_tx(tx_pin); // TX pin

        UART_INSTANCE = Some(uart);

        println!("UART Echo ready on GPIO16(RX)/GPIO17(TX) at 19200 baud");
    }
}

/// Loop function for task to execute.
fn loop_fn() {
    unsafe {
        if let Some(ref mut uart) = UART_INSTANCE {
            let mut buffer = [0u8; 1];

            // Try to read a byte (non-blocking)
            if uart.read_ready() {
                if let Ok(n) = uart.read(&mut buffer) {
                    if n == 0 {
                        return;
                    }
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

                    // Echo the byte back
                    if uart.write(&buffer[..n]).is_err() {
                        println!("Failed to echo byte!");
                    }
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

#[main]
fn main() -> ! {
    // Initialize Martos (including UART)
    init_system();

    // Add task to execute
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);

    // Start task manager
    TaskManager::start_task_manager();
}
