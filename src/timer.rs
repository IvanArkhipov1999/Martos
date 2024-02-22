use crate::connection;

/// Type for tick counting. It is signed for synchronization. It should be u128.
pub type TickType = i64;

/// The definition of the timers themselves.
pub struct Timer {
    /// Number of ticks in timer.
    tick_counter: TickType,
    /// Synchronization period in ticks.
    synchronization_period: TickType,
    /// Scale coefficient for local vote protocol.
    synchronization_scale: f64,
}

static mut TIMER: Timer = Timer {
    tick_counter: 0,
    synchronization_period: 5,
    synchronization_scale: 0.1,
};

impl Timer {
    /// Creates new timer.
    pub fn new(
        tick_counter: TickType,
        synchronization_period: TickType,
        synchronization_scale: f64,
    ) -> Timer {
        Timer {
            tick_counter,
            synchronization_period,
            synchronization_scale,
        }
    }

    pub fn setup_timer() {}

    /// Starts timer ticking.
    pub fn loop_timer() {
        unsafe {
            TIMER.tick_counter += 1;
        }
    }

    /// Stops timer ticking.
    pub fn stop_condition_timer() -> bool {
        if (unsafe { TIMER.tick_counter }) == 100 {
            return true;
        }
        return false;
    }

    /// Returns tick counter.
    pub fn get_tick_counter() -> TickType {
        return unsafe { TIMER.tick_counter };
    }

    // /// Synchronizes tick counter by information from other timers
    // fn synchronize(count: TickType, synchronization_scale: f64) {
    //     let timers_information = connection::get_timers_information();
    //     // Local vote protocol.
    //     let old_count = *count;
    //     for info in timers_information {
    //         *count +=
    //             (synchronization_scale * (old_count - info).abs() as f64).round() as TickType;
    //     }
    // }
}
