use crate::ports::xtensa_esp32::hardware_timer::TIMER10;
use crate::ports::xtensa_esp32::peripherals::PERIPHERALS_VARIABLE;
use esp_hal::rng::Rng;
use esp_wifi::{esp_now::EspNow, init, EspWifiInitFor};

pub static mut ESP_NOW: Option<EspNow> = None;

/// Network initialization.
pub fn init_network() {
    unsafe {
        if let Some(peripherals) = &mut PERIPHERALS_VARIABLE {
            let peripherals_rng = peripherals.rng.take().expect("RNG peripherals error");
            let peripherals_radio_clk = peripherals
                .radio_clk
                .take()
                .expect("RADIO_CLK peripherals error");
            let peripherals_wifi = peripherals.wifi.take().expect("WIFI peripherals error");

            let timer10 = TIMER10.take().expect("Network timer error");

            let init = init(
                EspWifiInitFor::Wifi,
                timer10,
                Rng::new(peripherals_rng),
                peripherals_radio_clk,
            )
                .unwrap();

            ESP_NOW = Some(esp_wifi::esp_now::EspNow::new(&init, peripherals_wifi).unwrap());
        }
    }
}

/// Getting esp-now object for network.
pub fn get_esp_now() -> EspNow<'static> {
    unsafe {
        let esp_now = ESP_NOW.take().expect("Esp-now error");
        return esp_now;
    }
}
