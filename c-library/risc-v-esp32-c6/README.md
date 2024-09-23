# Martos C static library

This README provides detailed instructions on how to build the Martos C static library independently, should you wish to do so.

## How to install dependencies

For comprehensive guidance on installing the necessary dependencies for developing applications targeting the RISC-V ESP32-C6 architecture,
please refer to [the official website](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html).
Below is an illustrative example demonstrating the installation of building toolchains on a Linux (Ubuntu/Debian):
```
cargo install espup
espup install
```

## How to build the library

For a thorough guide on developing projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html#3-set-up-the-environment-variables).
Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
. $HOME/export-esp.sh
cargo build
```
