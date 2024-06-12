use crate::timer::TickType;
use esp32_hal::timer::{Timer0, TimerGroup};
use esp32_hal::Timer;
use esp32_hal::{clock::ClockControl, peripherals::*, prelude::*};

static mut TIMER00: Option<Timer<Timer0<TIMG0>>> = None;

/// Esp32 hardware timer setup.
pub fn setup_hardware_timer() {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    let mut timer00 = timer_group0.timer0;
    timer00.start(500u64.millis());
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
