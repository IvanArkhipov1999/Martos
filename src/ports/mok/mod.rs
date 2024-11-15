pub mod hardware_timer;
pub mod memory_manager;
#[cfg(feature = "network")]
pub mod network;
use crate::ports::PortTrait;

/// PortTrait implementation for Mok platform
pub struct Mok;
impl PortTrait for Mok {
    fn init_heap() {
        memory_manager::init_heap();
    }

    fn setup_hardware_timer() {
        hardware_timer::setup_hardware_timer();
    }

    fn valid_timer_index(_timer_index: u8) -> bool {
        true
    }

    fn start_hardware_timer(timer_index: u8) {
        hardware_timer::start_hardware_timer();
    }

    fn set_reload_mode(timer_index: u8, auto_reload: bool) {
        hardware_timer::set_reload_mode(auto_reload);
    }

    fn change_period_timer(timer_index: u8, period: core::time::Duration) {
        hardware_timer::change_period_timer(period);
    }

    fn get_time(timer_index: u8) -> core::time::Duration {
        hardware_timer::get_time()
    }

    fn stop_hardware_timer(timer_index: u8) -> bool {
        false
    }
    #[cfg(feature = "network")]
    fn init_network() {
        network::init_network();
    }
}
