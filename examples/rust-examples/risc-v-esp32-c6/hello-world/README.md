# Rust example for risc-v esp32c6 architecture

Presented here is a straightforward 'Hello world' Rust example utilizing Martos.

Within the setup function, the phrase 'Setup hello world!' is printed once.
Additionally, within the loop function, the phrase 'Loop hello world!' along with the counter value is printed fifty times.

## How to install dependencies

For comprehensive guidance on installing the necessary dependencies for developing applications targeting the RISC-V ESP32-C6 architecture,
please refer to [the official website](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html).
Below is an illustrative example demonstrating the installation of building toolchains on a Linux (Ubuntu/Debian):
```
apt-get -qq update
apt-get install -y -q build-essential curl
curl https://sh.rustup.rs -sSf | sh -s -- -y
rustup target add riscv32imac-unknown-none-elf
cargo install espflash
```

Note: ESP32-C6 (RISC-V) builds use the stable Rust toolchain; the `esp` toolchain
installed by `espup` is only required for Xtensa targets (ESP32).

## How to build the example

For a thorough guide on developing projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html#risc-v-devices).
Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
cargo build
```

## How to run the example
For detailed instructions on running projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.espressif.com/projects/rust/book/getting-started/tooling/espflash.html).
Below, you will find an illustrative example showcasing the running on a Linux system (Ubuntu/Debian):
```
cargo run
```
