pub mod hardware_timer;
#[cfg(not(feature = "mips64_timer_tests"))]
pub mod memory_manager;
#[cfg(feature = "network")]
pub mod network;
use crate::ports::PortTrait;

/// PortTrait implementation for Mips64 platform
pub struct Mips64;
impl PortTrait for Mips64 {
    fn init_heap() {
        #[cfg(not(feature = "mips64_timer_tests"))]
        memory_manager::init_heap();
    }

    fn setup_hardware_timer() {
        hardware_timer::setup_hardware_timer();
    }

    fn valid_timer_index(timer_index: u8) -> bool {
        if timer_index <= 4 {
            true
        } else {
            false
        }
    }

    fn try_acquire_timer(timer_index: u8) -> bool {
        hardware_timer::try_acquire_timer(timer_index)
    }

    fn start_hardware_timer(timer_index: u8) {
        hardware_timer::start_hardware_timer(timer_index);
    }

    fn set_reload_mode(timer_index: u8, auto_reload: bool) {
        hardware_timer::set_reload_mode(timer_index, auto_reload);
    }

    fn change_period_timer(timer_index: u8, period: core::time::Duration) {
        hardware_timer::change_period_timer(timer_index, period);
    }

    fn get_time(timer_index: u8) -> core::time::Duration {
        hardware_timer::get_time(timer_index)
    }

    fn stop_hardware_timer(timer_index: u8) -> bool {
        hardware_timer::stop_hardware_timer(timer_index)
    }

    fn release_hardware_timer(timer_index: u8) {
        hardware_timer::release_hardware_timer(timer_index)
    }

    #[cfg(feature = "network")]
    fn init_network() {
        network::init_network();
    }
}
