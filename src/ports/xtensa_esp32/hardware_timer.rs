use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use esp_hal::timer::timg::{Timer, Timer0, TimerGroup};
use esp_hal::{peripherals::*, prelude::*};

// TODO: initialize peripherals in separate mod
pub static mut TIMER00: Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>> = None;
pub static mut PERIFERALS_RNG: Option<RNG> = None;
pub static mut PERIFERALS_RADIO_CLK: Option<RADIO_CLK> = None;
pub static mut PERIFERALS_WIFI: Option<WIFI> = None;

static TIMER_BUSY: AtomicBool = AtomicBool::new(false);

/// Esp32 hardware timer setup.
pub fn setup_hardware_timer() {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);

    let timer00 = timer_group0.timer0;
    let _ = timer00.load_value(500u64.millis());
    timer00.start();
    timer00.listen();

    unsafe {
        TIMER00 = Some(timer00);
        PERIFERALS_RNG = Some(peripherals.RNG);
        PERIFERALS_RADIO_CLK = Some(peripherals.RADIO_CLK);
        PERIFERALS_WIFI = Some(peripherals.WIFI);
    }
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
