use crate::timer::Timer;

pub mod timer;

fn main() {
    let my_timer = Timer {name: String::from("timer")};
    println!("MA-RTOS {:?}!", my_timer);
}
