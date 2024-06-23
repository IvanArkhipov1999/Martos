use crate::timer::TickType;
use esp_hal::timer::timg::{Timer, Timer0, TimerGroup};
use esp_hal::{clock::ClockControl, system::SystemControl, peripherals::*, prelude::*};

static mut TIMER00: Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>> = None;

/// Esp32 hardware timer setup.
pub fn setup_hardware_timer() {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);

    let mut timer00 = timer_group0.timer0;
    timer00.load_value(500u64.millis());
    timer00.start();
    timer00.listen();
    unsafe {
        TIMER00 = Some(timer00);
    }
}

/// Esp32 getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    unsafe {
        let timer00 = TIMER00.take().expect("Timer error");
        let tick_counter = timer00.now();
        TIMER00 = Some(timer00);
        tick_counter
    }
}
