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

    fn start_hardware_timer() {
        hardware_timer::start_hardware_timer();
    }

    fn set_reload_mode(auto_reload: bool) {
        hardware_timer::set_reload_mode(auto_reload);
    }

    fn change_period_timer(period: core::time::Duration) {
        hardware_timer::change_period_timer(period);
    }

    fn get_time() -> core::time::Duration {
        hardware_timer::get_time()
    }

    fn stop_hardware_timer() -> bool {
        false
    }
    #[cfg(feature = "network")]
    fn init_network() {
        network::init_network();
    }
}
