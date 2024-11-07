use core::time::Duration;

/// Mok hardware timer setup.
pub fn setup_hardware_timer() {}

/// Mok start harware timer.
pub fn start_hardware_timer() {}

/// Mok change the period of a timer.
pub fn change_period_timer(_period: Duration) {}

/// Mok getting counter value.
pub fn get_time() -> Duration {
    Duration::new(0, 0)
}
