# Rust example for mips64 architecture

Presented here is a straightforward Rust example utilizing Martos with timer usage.

## How to install dependencies

Below is an illustrative example demonstrating the installation of building toolchains on a Linux (Ubuntu/Debian):
```
apt update && apt install curl build-essential lld
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 
rustup toolchain install nightly 
rustup default 1.71
rustup target add mips64el-unknown-linux-gnuabi64 
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
```

## How to build the example

Below, you will find an illustrative example showcasing the building process on a Linux system (Ubuntu/Debian):
```
cargo +nightly build --release
```

## How to run the example

Below, you will find an illustrative example showcasing the running on a Linux system (Ubuntu/Debian):
```
cargo +nightly run
```
