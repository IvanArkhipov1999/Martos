use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

/// Type for tick counting.
type TickType = u128;

/// The definition of the timers themselves.
pub struct Timer {
    /// Number of ticks in timer.
    tick_counter: Arc<Mutex<TickType>>,
    /// Flag is timer running.
    running: Arc<Mutex<bool>>,
    /// Synchronization period in ticks.
    synchronization_period: TickType,
}

impl Timer {
    /// Creates new timer.
    pub fn new(tick_counter: TickType, synchronization_period: TickType) -> Timer {
        Timer { tick_counter: Arc::new(Mutex::new(tick_counter)), running: Arc::new(Mutex::new(false)), synchronization_period }
    }

    /// Starts timer ticking.
    // TODO: this ticking should be added as a privilege task in task manager. Now it is in a separate thread.
    pub fn start(&self) {
        let counter = self.tick_counter.clone();
        let running = self.running.clone();
        let synchronization_period = self.synchronization_period.clone();

        *running.lock().unwrap() = true;

        thread::spawn(move || {
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(1));
                let mut count = counter.lock().unwrap();
                // TODO: this ticking should work with hardware ticks or with system ticks, not '+1'
                *count += 1;
                if *count % synchronization_period == 0 {
                    Timer::synchronize(count)
                }
            }
        });
    }

    /// Stops timer ticking.
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    /// Returns tick counter.
    pub fn get_tick_counter(&self) -> TickType {
        *self.tick_counter.lock().unwrap()
    }

    #[warn(unused_mut)]
    /// Synchronizes tick counter by information from other timers
    fn synchronize(mut _count: MutexGuard<TickType>) {
        // Some synchronization code
        // *_count += 30;
    }
}
