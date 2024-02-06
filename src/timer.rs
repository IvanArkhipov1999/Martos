use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Type for tick counting.
type TickType = u32;

/// The definition of the timers themselves.
#[derive(Debug)]
pub struct Timer {
    /// Number of ticks in timer by now
    tick_counter: Arc<Mutex<TickType>>,
    /// Flag is timer running
    running: Arc<Mutex<bool>>,
}

impl Timer {
    /// Creates new timer.
    pub fn new(tick_counter: TickType) -> Timer {
        Timer { tick_counter: Arc::new(Mutex::new(tick_counter)), running: Arc::new(Mutex::new(false)) }
    }

    /// Starts timer ticking.
    /// TODO: this ticking should be added as a privilege task in task manager. Now it is in a separate thread.
    pub fn start(&self) {
        let counter = self.tick_counter.clone();
        let running = self.running.clone();

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(1));
                let mut count = counter.lock().unwrap();
                *count += 1;
            }
        });
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    pub fn get_count(&self) -> TickType {
        *self.tick_counter.lock().unwrap()
    }
}
