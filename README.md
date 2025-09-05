# Martos
![Martos ci workflow](https://github.com/IvanArkhipov1999/Martos/actions/workflows/rust.yml/badge.svg)
[![Version](https://img.shields.io/crates/v/martos.svg)](https://crates.io/crates/martos)
[![Release](https://img.shields.io/github/v/release/IvanArkhipov1999/Martos)](https://github.com/IvanArkhipov1999/Martos/releases)

Martos is a modern, lightweight real-time operating system (RTOS) written entirely in Rust, designed for performance, safety, and modularity in embedded systems. It offers a flexible task management system with support for both cooperative and preemptive scheduling, and is designed to be highly portable across different architectures.

## Key Features

- **Cooperative \& Preemptive Scheduling**: Choose the scheduling model that best fits your application's needs. Martos supports both cooperative round-robin scheduling and preemptive priority-based scheduling.
- **Multi-platform Support**: Martos is designed to be highly portable, with current support for:
    - **Xtensa**: ESP32, ESP32-S2, ESP32-S3
    - **RISC-V**: ESP32-C3, ESP32-C6
    - **MIPS64**: Initial support for MIPS64 architecture
    - **Mock**: A mock platform for testing and simulation
- **Modular Architecture**: Martos is designed with a modular architecture, allowing you to include only the features you need. This is achieved through Rust's feature flags, which enable you to include or exclude functionalities such as:
    - **Networking**: `network` feature enables ESP-NOW support
    - **Preemptive Scheduling**: `preemptive` feature enables preemptive multitasking
- **Hardware Abstraction Layer (HAL)**: Martos provides a clean hardware abstraction layer (HAL) that separates the core OS logic from platform-specific details. This makes it easy to port Martos to new architectures and platforms.


## Getting Started: Creating Your First Project

This guide will walk you through creating a simple "Hello, World!" application using Martos on an ESP32.

### 1. Set Up Your Project

Create a new binary crate for your application:

```bash
cargo new --bin my-martos-app
cd my-martos-app
```


### 2. Configure `Cargo.toml`

Add Martos and the necessary `esp-hal` dependencies to your `Cargo.toml`:

```toml
[package]
name = "my-martos-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Specify the path to your local Martos clone
martos = { path = "../Martos" } 
esp-hal = "0.21.1"
esp-backtrace = { version = "0.14.1", features = ["esp32", "panic-handler", "exception-handler", "println"] }
esp-println = { version = "0.11.0", features = ["esp32"] }

[features]
default = ["esp-hal/esp32", "esp-backtrace/esp32", "esp-println/esp32"]
```


### 3. Write Your Application (`src/main.rs`)

Martos applications are structured around tasks defined by `setup`, `loop`, and `stop_condition` functions.

```rust
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use esp_backtrace as _;
use esp_hal::entry;
use esp_println::println;
use martos::{
    init_system,
    task_manager::{TaskManager, TaskManagerTrait},
};

/// A counter to work with in the loop.
static COUNTER: AtomicU32 = AtomicU32::new(1);

/// The setup function for your task, runs once.
fn setup_fn() {
    println!("Setup: Hello from Martos!");
}

/// The main loop for your task, runs repeatedly.
fn loop_fn() {
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    println!("Loop: Hello from Martos! Counter = {}", count);
}

/// The stop condition for your task.
/// The task will stop when this function returns true.
fn stop_condition_fn() -> bool {
    let value = unsafe { COUNTER.as_ptr().read() };
    if value > 50 {
        println!("Task finished.");
        return true;
    }
    false
}

#[entry]
fn main() -> ! {
    // Initialize the Martos system.
    init_system();
    
    // Add your task to the Task Manager.
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);
    
    // Start the Task Manager. This call will not return.
    TaskManager::start_task_manager();
}
```


### 4. Build and Run

To build and flash your application to an ESP32, you can use `espflash`:

```bash
cargo build --release
espflash flash --monitor target/xtensa-esp32-none-elf/release/my-martos-app
```


## C Compatibility

Martos is designed to be compatible with C, allowing you to integrate it into existing C projects or expose its functionality to C code. This is achieved by compiling Martos as a static library (`.a` file) that can be linked with your C application.

### Exposing a C-Compatible API

To expose Martos functions to C, you can use the `#[no_mangle]` and `extern "C"` attributes. Martos provides a `c-library` feature flag and a `c_api.rs` module to facilitate this.

**Example of an exposed function in `c_api.rs`:**

```rust
#[no_mangle]
pub extern "C" fn martos_init() {
    init_system();
}

#[no_mangle]
pub extern "C" fn martos_start() {
    TaskManager::start_task_manager();
}
```


### Building as a Static Library

To compile Martos as a static library, add the following to your `Cargo.toml`:

```toml
[lib]
name = "martos"
crate-type = ["staticlib"]
```

Then, build the library using Cargo. This will produce a `libmartos.a` file that can be linked with your C project.

## Contributing

Contributions to Martos are welcome! Whether you're interested in adding support for a new platform, implementing a new feature, or fixing a bug, your contributions are valuable. Please feel free to open an issue or submit a pull request on the GitHub repository.

## License

Martos is licensed under the MIT License, making it suitable for a wide range of applications, including commercial products.
