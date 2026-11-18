# Martos C static library

This README provides detailed instructions on how to build the Martos C static library independently, should you wish to do so.

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

## How to build the library

For a thorough guide on developing projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html#risc-v-devices).
Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
cargo build
```
