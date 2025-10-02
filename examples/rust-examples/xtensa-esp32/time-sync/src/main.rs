//! ESP32 Time Synchronization Example
//!
//! This example demonstrates the Local Voting Protocol time synchronization
//! system running on ESP32. It shows how to set up and use the time
//! synchronization manager with ESP-NOW communication.
//!
//! # Overview
//!
//! The example implements a complete time synchronization system that:
//!
//! - Initializes ESP-NOW communication
//! - Sets up the time synchronization manager
//! - Sends periodic time broadcasts every 100ms
//! - Receives and processes time synchronization messages
//! - Applies Local Voting Protocol corrections
//! - Displays synchronization progress and offset information
//!
//! # Hardware Requirements
//!
//! - ESP32 development board
//! - USB cable for programming and monitoring
//!
//! # Usage
//!
//! 1. Flash this example to your ESP32
//! 2. Connect another ESP32 or ESP32-C6 running the same example
//! 3. Monitor serial output to see synchronization progress
//! 4. Observe how time differences decrease over time
//!
//! # Expected Output
//!
//! ```
//! ESP32: Setup time synchronization!
//! ESP32: Time synchronization setup complete!
//! ESP32: Received timestamp: 28290012μs, corrected time: 427750μs, diff: 27862262μs
//! ESP32: Current offset: 100000μs
//! ```
//!
//! # Configuration
//!
//! The synchronization parameters can be adjusted in the `SyncConfig`:
//!
//! - `sync_interval_ms`: How often to send sync messages (100ms)
//! - `max_correction_threshold_us`: Max correction per cycle (100000μs)
//! - `acceleration_factor`: Aggressiveness for large differences (0.8)
//! - `deceleration_factor`: Conservativeness for small differences (0.6)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{entry, time};
use esp_println::println;
use esp_wifi::esp_now::{EspNow, BROADCAST_ADDRESS};
use martos::get_esp_now;
use martos::{
    init_system,
    task_manager::{TaskManager, TaskManagerTrait},
    time_sync::{SyncConfig, SyncMessage, TimeSyncManager},
};

/// ESP-NOW communication instance for network operations
static mut ESP_NOW: Option<EspNow> = None;
/// Next scheduled time to send broadcast message (milliseconds)
static mut NEXT_SEND_TIME: Option<u64> = None;
/// Time synchronization manager instance
static mut SYNC_MANAGER: Option<TimeSyncManager<'static>> = None;

/// Setup function for time synchronization task.
///
/// This function initializes the ESP-NOW communication and sets up the
/// time synchronization manager with appropriate configuration parameters.
/// It configures the Local Voting Protocol with aggressive correction
/// factors for rapid convergence.
fn setup_fn() {
    println!("ESP32: Setup time synchronization!");
    unsafe {
        ESP_NOW = Some(get_esp_now());
        NEXT_SEND_TIME = Some(time::now().duration_since_epoch().to_millis() + 10);

        // Initialize time sync manager
        let esp_now = ESP_NOW.take().unwrap();
        let local_mac = [0x40, 0x4C, 0xCA, 0x57, 0x5A, 0xA4]; // ESP32 MAC
        let config = SyncConfig {
            node_id: 0x12345678,
            sync_interval_ms: 10,
            max_correction_threshold_us: 100000, // 100ms instead of 1ms
            acceleration_factor: 0.8,            // Much higher acceleration
            deceleration_factor: 0.6,            // Much higher deceleration
            max_peers: 10,
            adaptive_frequency: true,
        };
        let mut sync_manager = TimeSyncManager::new(config);
        sync_manager.init_esp_now_protocol(esp_now, local_mac);
        sync_manager.enable_sync();
        SYNC_MANAGER = Some(sync_manager);
    }
    println!("ESP32: Time synchronization setup complete!");
}

/// Main loop function for time synchronization task.
///
/// This function handles the continuous operation of the time synchronization
/// system. It processes incoming ESP-NOW messages, applies Local Voting Protocol
/// corrections, and sends periodic time broadcasts.
///
/// # Operations Performed
///
/// 1. **Message Reception**: Receives and processes ESP-NOW broadcast messages
/// 2. **Time Calculation**: Calculates time differences using corrected time
/// 3. **Synchronization**: Applies Local Voting Protocol corrections
/// 4. **Message Transmission**: Sends periodic time broadcasts every 100ms
/// 5. **Progress Display**: Shows synchronization progress and offset information
fn loop_fn() {
    unsafe {
        // Get ESP-NOW from sync_manager
        if let Some(ref mut sync_manager) = SYNC_MANAGER {
            // First, receive messages
            let received_message =
                if let Some(ref mut esp_now_protocol) = sync_manager.esp_now_protocol {
                    let esp_now = &mut esp_now_protocol.esp_now;
                    esp_now.receive()
                } else {
                    None
                };

            // Process received message
            if let Some(r) = received_message {
                // Process broadcast messages for time synchronization
                if r.info.dst_address == BROADCAST_ADDRESS {
                    // Try to create SyncMessage from received data
                    if let Some(received_sync_message) = SyncMessage::from_bytes(&r.data) {
                        let corrected_time_us = sync_manager.get_corrected_time_us();
                        let time_diff =
                            received_sync_message.timestamp_us as i64 - corrected_time_us as i64;
                        println!(
                            "ESP32: Received timestamp: {}μs, corrected time: {}μs, diff: {}μs",
                            received_sync_message.timestamp_us, corrected_time_us, time_diff
                        );

                        // Process message for synchronization
                        sync_manager.handle_sync_message(received_sync_message);
                    }
                }
            }

            // Send broadcast every 100ms
            let mut next_send_time = NEXT_SEND_TIME.take().expect("Next send time error in main");
            if time::now().duration_since_epoch().to_millis() >= next_send_time {
                next_send_time = time::now().duration_since_epoch().to_millis() + 10;

                // Create SyncMessage with corrected time
                let corrected_time_us = sync_manager.get_corrected_time_us();
                let sync_message = SyncMessage::new_sync_request(
                    0x12345678, // ESP32 node ID
                    0,          // broadcast
                    corrected_time_us,
                );
                let message_data = sync_message.to_bytes();

                if let Some(ref mut esp_now_protocol) = sync_manager.esp_now_protocol {
                    let esp_now = &mut esp_now_protocol.esp_now;
                    let _status = esp_now
                        .send(&BROADCAST_ADDRESS, &message_data)
                        .unwrap()
                        .wait();
                }
            }
            NEXT_SEND_TIME = Some(next_send_time);
        }
    }
}

/// Stop condition function for time synchronization task.
///
/// This function determines when the time synchronization task should stop.
/// In this example, it always returns `false`, meaning the task runs indefinitely
/// for continuous time synchronization.
///
/// # Returns
///
/// * `false` - Task continues running (infinite loop)
fn stop_condition_fn() -> bool {
    return false;
}

#[entry]
fn main() -> ! {
    // Initialize Martos.
    init_system();
    // Add task to execute.
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);
    // Start task manager.
    TaskManager::start_task_manager();
}
