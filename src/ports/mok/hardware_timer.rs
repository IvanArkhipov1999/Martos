use core::time::Duration;

/// Mok start harware timer.
pub fn start_hardware_timer() {}

/// Mok change operating mode of hardware timer.
pub fn set_reload_mode(_auto_reload: bool) {}

/// Mok change the period of hardware timer.
pub fn change_period_timer(_period: Duration) {}

/// Mok getting counter value of hardware timer.
pub fn get_time() -> Duration {
    Duration::new(0, 0)
}

/// Mok release hardware timer.
pub fn release_hardware_timer() {}
