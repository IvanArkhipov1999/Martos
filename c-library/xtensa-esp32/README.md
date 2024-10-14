# Martos C static library

This README provides detailed instructions on how to build the Martos C static library independently, should you wish to do so.

## How to install dependencies

For comprehensive guidance on installing the necessary dependencies for developing applications targeting the Xtensa ESP32 architecture,
please refer to [the official website](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html).
Below is an illustrative example demonstrating the installation of building toolchains on a Linux (Ubuntu/Debian):
```
apt-get -qq update
apt-get install -y -q build-essential curl
curl https://sh.rustup.rs -sSf | sh -s -- -y
cargo install espup
espup install
```

## How to build the library

For a thorough guide on developing projects for the Xtensa ESP32 architecture across various operating systems,
we recommend consulting [the official website](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html#3-set-up-the-environment-variables).
Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
. $HOME/export-esp.sh
cargo build
```
