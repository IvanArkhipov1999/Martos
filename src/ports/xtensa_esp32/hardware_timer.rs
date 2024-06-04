use crate::timer::TickType;
use esp32_hal::timer::TimerGroup;
use esp32_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*};

/// Esp32 hardware timer setup.
pub fn setup_hardware_timer() {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer00 = timer_group0.timer0;

    timer00.start(500u64.millis());
    timer00.listen();
}

/// Esp32 getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    0
}
