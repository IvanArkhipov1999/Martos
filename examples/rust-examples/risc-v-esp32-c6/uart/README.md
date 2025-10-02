# UART Echo Example for ESP32-C6

This example demonstrates UART communication on ESP32-C6 using the Martos RTOS framework.

## Features

- UART echo functionality
- Receives bytes via UART and echoes them back
- Uses GPIO16 (RX) and GPIO17 (TX) pins
- Baud rate: 19200
- Configuration: 8N1 (8 data bits, no parity, 1 stop bit)

## Hardware Setup

Connect your ESP32-C6 to a UART-to-USB converter:
- ESP32-C6 GPIO16 → UART-to-USB RX
- ESP32-C6 GPIO17 → UART-to-USB TX
- GND → GND

## Building and Flashing

Make sure you have the ESP toolchain installed:

```bash
# Install ESP toolchain
rustup toolchain install esp

# Set the toolchain for this project
rustup override set esp
```

Build the project:

```bash
cargo build --release
```

Flash to ESP32-C6:

```bash
cargo run --release
```

Or use espflash directly:

```bash
espflash flash target/riscv32imac-unknown-none-elf/release/example_risc_v_esp32c6
```

## Usage

1. Flash the firmware to your ESP32-C6
2. Connect a UART-to-USB converter to GPIO16/GPIO17
3. Open a serial terminal (e.g., minicom, screen, or Arduino IDE Serial Monitor)
4. Set the terminal to 19200 baud, 8N1
5. Type characters - they will be echoed back with additional information

## Expected Output

The example will print debug information via the ESP32-C6's built-in USB serial (if available) and echo received bytes via the external UART pins.

## Architecture Notes

This example is specifically configured for ESP32-C6 (RISC-V architecture):
- Uses UART0 peripheral (ESP32-C6 uses UART0 instead of UART2)
- Targets `riscv32imac-unknown-none-elf`
- Uses ESP32-C6 specific HAL features

## Troubleshooting

- Make sure your UART-to-USB converter is set to 19200 baud
- Check that GPIO16 and GPIO17 are not being used by other peripherals
- Verify that the ESP32-C6 is properly powered and grounded