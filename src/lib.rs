use std::thread;
use std::time::Duration;
use lazy_static::lazy_static;
use crate::timer::Timer;

pub mod connection;
pub mod timer;

lazy_static! {
static ref TIMER: Timer = Timer::new(0, 5, 0.1);
}

#[no_mangle]
pub extern "C" fn init_timer() {
    TIMER.start();
    println!("Ticks time: {}", TIMER.get_tick_counter());
    thread::sleep(Duration::from_millis(5));
    println!("Ticks time: {}", TIMER.get_tick_counter());
    TIMER.stop();
}
