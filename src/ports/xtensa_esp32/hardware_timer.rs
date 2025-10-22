use super::peripherals::{PERIFERALS_TIMG0, PERIFERALS_TIMG1};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use esp_hal::timer::timg::{Timer, Timer0, TimerGroup};
use esp_hal::{peripherals::*, prelude::*};

// Timer instances derived from stored raw peripherals
pub static mut TIMER00: Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>> = None;
pub static mut TIMER10: Option<Timer<Timer0<TIMG1>, esp_hal::Blocking>> = None;

static TIMER_BUSY: AtomicBool = AtomicBool::new(false);

/// Esp32 hardware timer setup.
pub fn setup_hardware_timer() {
    let (timg0, timg1) = unsafe {
        (
            PERIFERALS_TIMG0.take().expect("TIMG0 peripherals error"),
            PERIFERALS_TIMG1.take().expect("TIMG1 peripherals error"),
        )
    };
    let timer_group0 = TimerGroup::new(timg0);
    let timer_group1 = TimerGroup::new(timg1);

    let timer00 = timer_group0.timer0;
    let timer10 = timer_group1.timer0;

    unsafe {
        TIMER00 = Some(timer00);
        TIMER10 = Some(timer10);
    }
}

/// Esp32 attempt to acquire timer.
pub fn try_acquire_timer(timer_index: u8) -> bool {
    match timer_index {
        0 => {
            match TIMER_BUSY.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        1 => {
            // Timer 1 (TIMER10) is always available for time measurement
            // It's not used by the scheduler, only by network (when enabled)
            true
        }
        _ => false,
    }
}

/// Esp32 start harware timer.
pub fn start_hardware_timer(timer_index: u8) {
    match timer_index {
        0 => {
            // Timer 0 is managed by the preemptive scheduler
            // Do nothing here as it's already started
        }
        1 => {
            // Timer 1 can be started for time measurement
            unsafe {
                if let Some(timer) = TIMER10.take() {
                    timer.start();
                    TIMER10 = Some(timer);
                }
            }
        }
        _ => {}
    }
}

/// Esp32 change operating mode of hardware timer.
pub fn set_reload_mode(timer_index: u8, auto_reload: bool) {
    match timer_index {
        0 => {
            // Timer 0 is managed by the preemptive scheduler
            // Do nothing here as it's configured by the scheduler
        }
        1 => {
            // Timer 1 can be configured for time measurement
            unsafe {
                if let Some(timer) = TIMER10.take() {
                    if auto_reload {
                        timer.set_auto_reload(true);
                    } else {
                        timer.set_auto_reload(false);
                    }
                    TIMER10 = Some(timer);
                }
            }
        }
        _ => {}
    }
}

/// Esp32 change the period of hardware timer.
pub fn change_period_timer(timer_index: u8, period: Duration) {
    match timer_index {
        0 => {
            // Timer 0 is managed by the preemptive scheduler
            // Do nothing here as it's configured by the scheduler
        }
        1 => {
            // Timer 1 can be configured for time measurement
            unsafe {
                if let Some(timer) = TIMER10.take() {
                    // Convert core::time::Duration to ESP32 HAL Duration
                    let micros = period.as_micros() as u64;
                    timer.load_value(micros.micros()).unwrap();
                    TIMER10 = Some(timer);
                }
            }
        }
        _ => {}
    }
}

/// Esp32 getting counter value of hardware timer.
pub fn get_time(timer_index: u8) -> Duration {
    match timer_index {
        0 => {
            unsafe {
                let timer00 = TIMER00.take().expect("Timer error");
                let tick_counter = timer00.now();
                TIMER00 = Some(timer00);
                Duration::from_micros(tick_counter.ticks())
            }
        }
        1 => {
            unsafe {
                let timer10 = TIMER10.take().expect("Timer error");
                let tick_counter = timer10.now();
                TIMER10 = Some(timer10);
                Duration::from_micros(tick_counter.ticks())
            }
        }
        _ => Duration::from_micros(0),
    }
}

/// Esp32 release hardware timer.
pub fn release_hardware_timer(timer_index: u8) {
    match timer_index {
        0 => {
            TIMER_BUSY.store(false, Ordering::Release);
        }
        1 => {
            // Timer 1 doesn't need explicit release as it's always available
        }
        _ => {}
    }
}
