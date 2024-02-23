use ma_rtos::timer::Timer;

fn main() {
    println!("Timer {}", Timer::get_tick_counter());
}