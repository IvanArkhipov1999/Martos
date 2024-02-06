use crate::timer::Timer;

use std::thread;
use std::time::Duration;

pub mod timer;

fn main() {
    let my_timer = Timer::new(String::from("timer"), 0, 20);
    Timer::start_timer(Box::leak(Box::new(my_timer)));

    thread::sleep(Duration::from_millis(7));

    // println!("MA-RTOS {:?}!", my_timer);
}
