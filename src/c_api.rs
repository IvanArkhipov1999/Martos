use crate::{task_manager, timer};
use core::time::Duration;
use task_manager::{TaskManager, TaskManagerTrait};
use timer::Timer;

/// The structure represents duration in seconds and microseconds.
/// It is used to pass time intervals between programming languages.
#[repr(C)]
pub struct DurationFFI {
    secs: u64,
    micros: u32,
}

/// The structure is used to return information about a timer.
#[repr(C)]
pub struct TimerOption {
    /// Indicator whether the timer exists.
    is_some: bool,
    /// The timer itself.
    timer: Timer,
}

#[no_mangle]
pub extern "C" fn init_system() {
    super::init_system();
}

#[no_mangle]
pub extern "C" fn setup_timer() {
    Timer::setup_timer();
}

#[no_mangle]
pub extern "C" fn get_timer(timer_index: u8) -> TimerOption {
    if let Some(timer) = Timer::get_timer(timer_index) {
        TimerOption {
            is_some: true,
            timer,
        }
    } else {
        TimerOption {
            is_some: false,
            timer: Timer {
                timer_index: 0,
                tick_counter: 0,
                sync_offset_us: 0,
            },
        }
    }
}

#[no_mangle]
pub extern "C" fn start_timer(timer: &Timer) {
    Timer::start_timer(timer);
}

#[no_mangle]
pub extern "C" fn set_reload_mode(timer: &Timer, auto_reload: bool) {
    Timer::set_reload_mode(timer, auto_reload);
}

#[no_mangle]
pub extern "C" fn change_period_timer(timer: &Timer, period: DurationFFI) {
    Timer::change_period_timer(timer, Duration::new(period.secs, period.micros));
}

#[no_mangle]
pub extern "C" fn loop_timer(timer: &mut Timer) {
    Timer::loop_timer(timer);
}

#[no_mangle]
pub extern "C" fn get_time(timer: &Timer) -> DurationFFI {
    let time = Timer::get_time(timer);
    DurationFFI {
        secs: time.as_secs(),
        micros: time.subsec_micros(),
    }
}

#[no_mangle]
pub extern "C" fn stop_condition_timer(timer: &Timer) -> bool {
    Timer::stop_condition_timer(timer)
}

#[no_mangle]
pub extern "C" fn release_timer(timer: &Timer) {
    Timer::release_timer(timer)
}

#[no_mangle]
pub extern "C" fn add_task(
    setup_fn: extern "C" fn() -> (),
    loop_fn: extern "C" fn() -> (),
    stop_condition_fn: extern "C" fn() -> bool,
) {
    TaskManager::add_task(setup_fn, loop_fn, stop_condition_fn)
}

#[no_mangle]
pub extern "C" fn start_task_manager() {
    TaskManager::start_task_manager()
}
