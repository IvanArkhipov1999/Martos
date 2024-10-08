# Rust example for risc-v esp32c6 architecture

Presented here is a straightforward Rust example utilizing Martos with dynamic memory usage.

Within the setup function, the phrase 'Setup hello world!' is printed once.
Additionally, within the loop function, the phrase 'Loop hello world!' along with the counter value (last value of dynamic vector) is printed fifty times.

## How to install dependencies

For comprehensive guidance on installing the necessary dependencies for developing applications targeting the RISC-V ESP32-C6 architecture,
please refer to [the official website](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html).
Below is an illustrative example demonstrating the installation of building toolchains on a Linux (Ubuntu/Debian):
```
apt-get -qq update
apt-get install -y -q build-essential curl
curl https://sh.rustup.rs -sSf | sh -s -- -y
cargo install espup
espup install
```

## How to build the example

For a thorough guide on developing projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html#3-set-up-the-environment-variables).
Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
. $HOME/export-esp.sh
cargo build
```

## How to run the example
For detailed instructions on running projects for the RISC-V ESP32-C6 architecture across various operating systems,
we recommend consulting [the official website](https://docs.esp-rs.org/book/tooling/espflash.html).
Below, you will find an illustrative example showcasing the running on a Linux system (Ubuntu/Debian):
```
cargo run
```
