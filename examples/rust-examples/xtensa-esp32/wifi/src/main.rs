#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::entry;
use esp_println::println;
use esp_wifi::{
    current_millis,
    esp_now::{EspNow, PeerInfo, BROADCAST_ADDRESS},
};
use martos::get_esp_now;
use martos::init_system;
use martos::task_manager::TaskManager;

/// Esp-now object for network
static mut ESP_NOW: Option<EspNow> = None;
/// Variable for saving time to send broadcast message
static mut NEXT_SEND_TIME: Option<u64> = None;

/// Setup function for task to execute.
fn setup_fn() {
    println!("Setup hello world!");
    unsafe {
        ESP_NOW = Some(get_esp_now());
        NEXT_SEND_TIME = Some(current_millis() + 5 * 1000);
    }
}

/// Loop function for task to execute.
fn loop_fn() {
    unsafe {
        let mut esp_now = ESP_NOW.take().expect("Esp-now error in main");

        let r = esp_now.receive();
        if let Some(r) = r {
            println!("Received {:?}", r);

            if r.info.dst_address == BROADCAST_ADDRESS {
                if !esp_now.peer_exists(&r.info.src_address) {
                    esp_now
                        .add_peer(PeerInfo {
                            peer_address: r.info.src_address,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }
                let status = esp_now
                    .send(&r.info.src_address, b"Hello Peer")
                    .unwrap()
                    .wait();
                println!("Send hello to peer status: {:?}", status);
            }
        }

        let mut next_send_time = NEXT_SEND_TIME.take().expect("Next send time error in main");
        if current_millis() >= next_send_time {
            next_send_time = current_millis() + 5 * 1000;
            println!("Send");
            let status = esp_now
                .send(&BROADCAST_ADDRESS, b"0123456789")
                .unwrap()
                .wait();
            println!("Send broadcast status: {:?}", status)
        }

        NEXT_SEND_TIME = Some(next_send_time);
        ESP_NOW = Some(esp_now);
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
