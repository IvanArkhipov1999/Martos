#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{entry, time};
use esp_println::println;
use martos::task_manager::{TaskManager, TaskManagerTrait};
use martos::time_sync::{SyncConfig, SyncPeer, TimeSyncManager};
use martos::timer::Timer;
use martos::{get_esp_now, init_system};

/// Time synchronization example for ESP32
///
/// This example demonstrates how to use the time synchronization system
/// with ESP-NOW communication. It shows:
/// - Setting up time synchronization
/// - Adding peers for synchronization
/// - Processing synchronization cycles
/// - Monitoring synchronization quality

/// Global variables for the application
static mut SYNC_MANAGER: Option<TimeSyncManager> = None;
static mut TIMER: Option<Timer> = None;
static mut ESP_NOW: Option<esp_wifi::esp_now::EspNow<'static>> = None;
static mut NEXT_SYNC_TIME: Option<u32> = None;
static mut NEXT_STATS_TIME: Option<u32> = None;

/// Setup function for the time synchronization task
fn setup_fn() {
    println!("=== ESP32 Time Synchronization Example ===");

    // Initialize Martos system
    init_system();

    // Get ESP-NOW instance
    unsafe {
        ESP_NOW = Some(get_esp_now());
    }

    // Create timer instance
    unsafe {
        TIMER = Timer::get_timer(0);
        if TIMER.is_none() {
            println!("ERROR: Failed to acquire timer 0");
            return;
        }

        // Configure timer for 1ms periodic interrupts
        let timer = TIMER.as_mut().unwrap();
        timer.set_reload_mode(true);
        timer.change_period_timer(core::time::Duration::from_millis(1));
        timer.start_timer();
    }

    // Create time synchronization manager
    let sync_config = SyncConfig {
        node_id: 0x12345678,               // Unique node ID
        sync_interval_ms: 2000,            // Sync every 2 seconds
        max_correction_threshold_us: 1000, // Max 1ms correction
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 10,
        adaptive_frequency: true,
    };

    unsafe {
        SYNC_MANAGER = Some(TimeSyncManager::new(sync_config));

        // Initialize ESP-NOW protocol handler
        if let Some(ref mut sync_manager) = SYNC_MANAGER {
            if let Some(esp_now) = ESP_NOW.take() {
                // Get local MAC address (simplified - in real app you'd get actual MAC)
                let local_mac = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
                sync_manager.init_esp_now_protocol(esp_now, local_mac);
            }

            // Add some example peers (in real app, peers would be discovered dynamically)
            let peer1 = SyncPeer::new(0x11111111, [0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);
            let peer2 = SyncPeer::new(0x22222222, [0x22, 0x22, 0x22, 0x22, 0x22, 0x22]);

            sync_manager.add_peer(peer1);
            sync_manager.add_peer(peer2);

            // Enable synchronization
            sync_manager.enable_sync();
        }

        // Set initial sync and stats times
        let current_time = time::now().duration_since_epoch().to_millis() as u32;
        NEXT_SYNC_TIME = Some(current_time + 5000); // First sync in 5 seconds
        NEXT_STATS_TIME = Some(current_time + 10000); // First stats in 10 seconds
    }

    println!("Time synchronization setup complete!");
    println!("Node ID: 0x{:08X}", 0x12345678);
    println!("Sync interval: 2000ms");
    println!("Max correction: 1000μs");
}

/// Loop function for the time synchronization task
fn loop_fn() {
    unsafe {
        let current_time = time::now().duration_since_epoch().to_millis() as u32;

        // Process timer tick
        if let Some(ref mut timer) = TIMER {
            timer.loop_timer();
        }

        // Process synchronization cycle
        if let Some(ref mut sync_manager) = SYNC_MANAGER {
            if let Some(next_sync_time) = NEXT_SYNC_TIME {
                if current_time >= next_sync_time {
                    // Process synchronization with ESP-NOW
                    let current_time_us = time::now().duration_since_epoch().to_micros() as u32;
                    sync_manager.process_sync_cycle_with_esp_now(current_time_us);

                    // Schedule next sync
                    NEXT_SYNC_TIME = Some(current_time + 2000);
                }
            }
        }

        // Print synchronization statistics
        if let Some(ref sync_manager) = SYNC_MANAGER {
            if let Some(next_stats_time) = NEXT_STATS_TIME {
                if current_time >= next_stats_time {
                    print_sync_stats(sync_manager);
                    NEXT_STATS_TIME = Some(current_time + 10000); // Stats every 10 seconds
                }
            }
        }

        // Demonstrate synchronized time usage
        if let Some(ref timer) = TIMER {
            if timer.tick_counter % 1000 == 0 {
                // Every 1000 ticks (1 second)
                let local_time = timer.get_time();
                let sync_time = timer.get_synchronized_time();
                let offset = timer.get_sync_offset_us();

                println!(
                    "Tick: {}, Local: {:?}, Sync: {:?}, Offset: {}μs",
                    timer.tick_counter, local_time, sync_time, offset
                );
            }
        }
    }
}

/// Print synchronization statistics
fn print_sync_stats(sync_manager: &TimeSyncManager) {
    println!("\n=== Synchronization Statistics ===");
    println!("Sync enabled: {}", sync_manager.is_sync_enabled());
    println!("Sync quality: {:.2}", sync_manager.get_sync_quality());
    println!("Time offset: {}μs", sync_manager.get_time_offset_us());

    let peers = sync_manager.get_peers();
    println!("Active peers: {}", peers.len());

    for peer in peers {
        println!(
            "  Peer 0x{:08X}: quality={:.2}, diff={}μs, syncs={}",
            peer.node_id, peer.quality_score, peer.time_diff_us, peer.sync_count
        );
    }

    // Print algorithm statistics if available
    #[cfg(feature = "network")]
    if let Some(stats) = sync_manager.get_sync_stats() {
        println!("Algorithm stats:");
        println!("  Avg time diff: {:.1}μs", stats.avg_time_diff_us);
        println!("  Max time diff: {}μs", stats.max_time_diff_us);
        println!("  Min time diff: {}μs", stats.min_time_diff_us);
        println!("  Current correction: {}μs", stats.current_correction_us);
        println!("  Converged: {}", stats.is_converged);
    }

    println!("=====================================\n");
}

/// Stop condition function (never stops in this example)
fn stop_condition_fn() -> bool {
    false
}

/// Main entry point
#[entry]
fn main() -> ! {
    // Initialize Martos system
    init_system();

    // Add time synchronization task
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn);

    // Start task manager
    TaskManager::start_task_manager();
}
