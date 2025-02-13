use crate::ports::xtensa_esp32::peripherals::{
    init_peripherals, PERIFERALS_RADIO_CLK, PERIFERALS_RNG, PERIFERALS_WIFI, TIMER00, TIMER10,
};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use esp_hal::timer::timg::{Timer, Timer0, TimerGroup};
use esp_hal::{peripherals::*, prelude::*};

static TIMER_BUSY: AtomicBool = AtomicBool::new(false);
pub fn setup_hardware_timer() {
    init_peripherals();
}

/// Esp32 attempt to acquire timer.
pub fn try_acquire_timer() -> bool {
    match TIMER_BUSY.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Esp32 start harware timer.
pub fn start_hardware_timer() {}

/// Esp32 change operating mode of hardware timer.
pub fn set_reload_mode(_auto_reload: bool) {}

/// Esp32 change the period of hardware timer.
pub fn change_period_timer(_period: Duration) {}

/// Esp32 getting counter value of hardware timer.
pub fn get_time() -> Duration {
    unsafe {
        let timer00 = TIMER00.take().expect("Timer error");
        let tick_counter = timer00.now();
        TIMER00 = Some(timer00);
        Duration::from_micros(tick_counter.ticks())
    }
}

/// Esp32 release hardware timer.
pub fn release_hardware_timer() {
    TIMER_BUSY.store(false, Ordering::Release);
}
