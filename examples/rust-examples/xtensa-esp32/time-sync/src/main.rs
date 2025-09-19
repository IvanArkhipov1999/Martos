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
    time_sync::{TimeSyncManager, SyncConfig},
};

/// Esp-now object for network
static mut ESP_NOW: Option<EspNow> = None;
/// Variable for saving time to send broadcast message
static mut NEXT_SEND_TIME: Option<u64> = None;
/// Time synchronization manager
static mut SYNC_MANAGER: Option<TimeSyncManager<'static>> = None;

/// Setup function for task to execute.
fn setup_fn() {
    println!("ESP32: Setup time synchronization!");
    unsafe {
        ESP_NOW = Some(get_esp_now());
        NEXT_SEND_TIME = Some(time::now().duration_since_epoch().to_millis() + 2000);
        
        // Initialize time sync manager
        let esp_now = ESP_NOW.take().unwrap();
        let local_mac = [0x40, 0x4C, 0xCA, 0x57, 0x5A, 0xA4]; // ESP32 MAC
        let config = SyncConfig {
            node_id: 0x12345678,
            sync_interval_ms: 2000,
            max_correction_threshold_us: 1000,
            acceleration_factor: 0.1,
            deceleration_factor: 0.05,
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

/// Loop function for task to execute.
fn loop_fn() {
    unsafe {
        // Получаем ESP-NOW из sync_manager
        if let Some(ref mut sync_manager) = SYNC_MANAGER {
            if let Some(ref mut esp_now_protocol) = sync_manager.esp_now_protocol {
                let esp_now = &mut esp_now_protocol.esp_now;
                
                // Получаем сообщения как в wifi примерах
                let r = esp_now.receive();
                if let Some(r) = r {
                    println!("ESP32: Received {:?}", r);
                    
                    // Просто получаем broadcast сообщения, никаких ответов не отправляем
                    if r.info.dst_address == BROADCAST_ADDRESS {
                        println!("ESP32: Received broadcast message from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", 
                            r.info.src_address[0], r.info.src_address[1], r.info.src_address[2],
                            r.info.src_address[3], r.info.src_address[4], r.info.src_address[5]);
                        println!("ESP32: Data: {:?}", r.data);
                    }
                }
                
                // Отправляем broadcast каждые 2 секунды как в wifi примерах
                let mut next_send_time = NEXT_SEND_TIME.take().expect("Next send time error in main");
                if time::now().duration_since_epoch().to_millis() >= next_send_time {
                    next_send_time = time::now().duration_since_epoch().to_millis() + 2000;
                    println!("ESP32: Send");
                    let status = esp_now
                        .send(&BROADCAST_ADDRESS, b"Time Sync Request")
                        .unwrap()
                        .wait();
                    println!("ESP32: Send broadcast status: {:?}", status);
                }
                
                NEXT_SEND_TIME = Some(next_send_time);
            }
            
            // Обработка синхронизации времени
            let current_time_us = time::now().duration_since_epoch().to_micros() as u32;
            sync_manager.process_sync_cycle_with_esp_now(current_time_us);
        }
    }
}

/// Stop condition function for task to execute.
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