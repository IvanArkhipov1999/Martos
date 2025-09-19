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
    println!("ESP32-C6: Setup time synchronization!");
    unsafe {
        ESP_NOW = Some(get_esp_now());
        NEXT_SEND_TIME = Some(time::now().duration_since_epoch().to_millis() + 2000);
        
        // Initialize time sync manager
        let esp_now = ESP_NOW.take().unwrap();
        let local_mac = [0x24, 0xDC, 0xC3, 0x9F, 0xD3, 0xD0]; // ESP32-C6 MAC
        let config = SyncConfig {
            node_id: 0x87654321,
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
    println!("ESP32-C6: Time synchronization setup complete!");
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
                    
                    // Парсим время из ESP-NOW сообщения
                    let current_time_us = time::now().duration_since_epoch().to_micros() as u64;
                    
                    // Пытаемся создать SyncMessage из полученных данных
                    if let Some(received_sync_message) = SyncMessage::from_bytes(&r.data) {
                        println!("ESP32-C6: Received timestamp: {}μs, current time: {}μs", received_sync_message.timestamp_us, current_time_us);
                        
                        // Обрабатываем сообщение для синхронизации
                        sync_manager.handle_sync_message(received_sync_message);
                    }
                }
            }
            
            // Отправляем broadcast каждые 2 секунды
            if let Some(ref mut esp_now_protocol) = sync_manager.esp_now_protocol {
                let esp_now = &mut esp_now_protocol.esp_now;
                let mut next_send_time = NEXT_SEND_TIME.take().expect("Next send time error in main");
                        if time::now().duration_since_epoch().to_millis() >= next_send_time {
                            next_send_time = time::now().duration_since_epoch().to_millis() + 2000;
                            
                            // Создаем правильное SyncMessage с текущим временем
                            let current_time_us = time::now().duration_since_epoch().to_micros() as u64;
                            let sync_message = SyncMessage::new_sync_request(
                                0x87654321, // ESP32-C6 node ID
                                0, // broadcast
                                current_time_us
                            );
                            let message_data = sync_message.to_bytes();
                            
                            let _status = esp_now
                                .send(&BROADCAST_ADDRESS, &message_data)
                                .unwrap()
                                .wait();
                        }
                NEXT_SEND_TIME = Some(next_send_time);
            }
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