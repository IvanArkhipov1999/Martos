use std::thread;
use std::time::Duration;

/// Type for tick counting.
type TickType = u32;

/// The definition of the timers themselves.
#[derive(Debug)]
pub struct Timer {
    /// Name of timer. It may be useful for debugging and users.
    name: String,
    /// Number of ticks in timer by now
    now_time: TickType,
    /// Number of ticks, after what timer will be expired.
    expired_time: TickType,
}

impl Timer {
    /// Creates new timer.
    pub fn new(name: String, now_time: TickType, expired_time: TickType) -> Timer {
        Timer { name, now_time, expired_time }
    }

    /// Starts timer ticking.
    /// TODO: this ticking should be added as a privilege task in task manager. Now it is in a separate thread.
    pub fn start_timer(timer: &'static mut Timer) {
        thread::spawn(move || {
            loop {
                timer.now_time += 1;
                println!("tick {} from the spawned thread!", timer.now_time);
                thread::sleep(Duration::from_millis(1));
            }
        });
    }
}
