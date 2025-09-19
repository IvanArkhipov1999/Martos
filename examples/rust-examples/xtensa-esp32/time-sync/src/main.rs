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
    time_sync::{TimeSyncManager, SyncConfig, SyncMessage},
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
            max_correction_threshold_us: 100000, // 100ms instead of 1ms
            acceleration_factor: 0.8,           // Much higher acceleration
            deceleration_factor: 0.6,           // Much higher deceleration
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
            // Сначала получаем сообщения
            let received_message = if let Some(ref mut esp_now_protocol) = sync_manager.esp_now_protocol {
                let esp_now = &mut esp_now_protocol.esp_now;
                esp_now.receive()
            } else {
                None
            };
            
            // Обрабатываем полученное сообщение
            if let Some(r) = received_message {
                // Обрабатываем broadcast сообщения для синхронизации времени
                if r.info.dst_address == BROADCAST_ADDRESS {
                    // Пытаемся создать SyncMessage из полученных данных
                    if let Some(received_sync_message) = SyncMessage::from_bytes(&r.data) {
                        let corrected_time_us = sync_manager.get_corrected_time_us();
                        let time_diff = received_sync_message.timestamp_us as i64 - corrected_time_us as i64;
                        println!("ESP32: Received timestamp: {}μs, corrected time: {}μs, diff: {}μs", received_sync_message.timestamp_us, corrected_time_us, time_diff);
                        
                        // Обрабатываем сообщение для синхронизации
                        sync_manager.handle_sync_message(received_sync_message);
                        
                        // Показываем текущий offset
                        let offset = sync_manager.get_time_offset_us();
                        println!("ESP32: Current offset: {}μs", offset);
                    }
                }
            }
            
            // Отправляем broadcast каждые 2 секунды
            let mut next_send_time = NEXT_SEND_TIME.take().expect("Next send time error in main");
            if time::now().duration_since_epoch().to_millis() >= next_send_time {
                next_send_time = time::now().duration_since_epoch().to_millis() + 2000;
                
                // Создаем SyncMessage с скорректированным временем
                let corrected_time_us = sync_manager.get_corrected_time_us();
                let sync_message = SyncMessage::new_sync_request(
                    0x12345678, // ESP32 node ID
                    0, // broadcast
                    corrected_time_us
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