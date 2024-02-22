use crate::timer::{TickType, Timer};
use lazy_static::lazy_static;
use std::thread;
use std::time::Duration;

pub mod connection;
pub mod task_manager;
pub mod timer;

// lazy_static! {
//     static ref TIMER: Timer = Timer::new(0, 5, 0.1);
// }
//
// #[no_mangle]
// pub extern "C" fn init_timer() {
//     TIMER.start();
//     thread::sleep(Duration::from_millis(5));
// }
//
// #[no_mangle]
// pub extern "C" fn stop_timer() {
//     thread::sleep(Duration::from_millis(5));
//     TIMER.stop();
// }
//
// #[no_mangle]
// pub extern "C" fn get_tick_counter() -> TickType {
//     TIMER.get_tick_counter()
// }
