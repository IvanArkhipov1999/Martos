use crate::timer::Timer;

use std::thread;
use std::time::Duration;

pub mod timer;

fn main() {
    let timer = Timer::new(0, 5);

    timer.start();
    println!("Ticks time: {}", timer.get_tick_counter());
    thread::sleep(Duration::from_millis(5));
    println!("Ticks time: {}", timer.get_tick_counter());
    thread::sleep(Duration::from_millis(3));
    timer.stop();

    println!("Final ticks time: {}", timer.get_tick_counter());
}

