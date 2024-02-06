use crate::timer::Timer;

pub mod timer;

fn main() {
    let my_timer = Timer {name: String::from("timer"), expired_time: 20};
    println!("MA-RTOS {:?}!", my_timer.name);
}
