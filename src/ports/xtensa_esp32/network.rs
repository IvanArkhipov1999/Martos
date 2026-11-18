use crate::ports::xtensa_esp32::hardware_timer::TIMER10;
use crate::ports::xtensa_esp32::peripherals::{PERIFERALS_FROM_CPU_INTR0, PERIFERALS_WIFI};
use esp_radio::esp_now::EspNow;
use esp_radio::wifi::WifiController;

pub static mut ESP_NOW: Option<EspNow> = None;

/// Network initialization.
///
/// esp-radio requires a running preemptive scheduler, so esp-rtos is started
/// first: it owns context switching and schedules the radio tasks, while
/// Martos tasks run cooperatively inside its main task.
pub fn init_network() {
    unsafe {
        let timer10 = TIMER10.take().expect("Network timer error");
        let from_cpu_intr0 = PERIFERALS_FROM_CPU_INTR0
            .take()
            .expect("FROM_CPU_INTR0 peripherals error");
        let peripherals_wifi = PERIFERALS_WIFI.take().expect("WIFI peripherals error");

        esp_rtos::start(timer10, from_cpu_intr0);

        let controller = WifiController::new(peripherals_wifi, Default::default())
            .expect("Wi-Fi controller init failed");
        // The ESP-NOW instance keeps Wi-Fi initialized on its own,
        // so the controller does not need to be stored.
        ESP_NOW = Some(controller.esp_now());
    }
}

/// Getting esp-now object for network.
pub fn get_esp_now() -> EspNow {
    unsafe {
        let esp_now = ESP_NOW.take().expect("Esp-now error");
        return esp_now;
    }
}
