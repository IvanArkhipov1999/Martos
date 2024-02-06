use crate::timer::Timer;

pub mod timer;

fn main() {
    let my_timer = Timer::new(String::from("timer"), 0, 20);
    println!("MA-RTOS {:?}!", my_timer);
}
