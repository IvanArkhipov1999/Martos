use crate::timer::Timer;

pub mod timer;

fn main() {
    let my_timer = Timer {name: String::from("timer"), now_time: 0, expired_time: 20};
    println!("MA-RTOS {:?}!", my_timer.name);
}
