use crate::timer::Timer;

pub mod timer;

fn main() {
    let my_timer = Timer {};
    println!("MA-RTOS {:?}!", my_timer);
}
