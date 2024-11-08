pub mod hardware_timer;
pub mod memory_manager;
use crate::ports::PortTrait;

/// PortTrait implementation for Mips64 platform
pub struct Mips64;
impl PortTrait for Mips64 {
    fn init_heap() {
        memory_manager::init_heap();
    }

    fn setup_hardware_timer() {
        hardware_timer::setup_hardware_timer();
    }

    fn start_hardware_timer() {
        hardware_timer::start_hardware_timer();
    }

    fn change_period_timer(period: core::time::Duration) {
        hardware_timer::change_period_timer(period);
    }

    fn get_time() -> core::time::Duration {
        hardware_timer::get_time()
    }

    fn stop_hardware_timer() -> bool {
        hardware_timer::stop_hardware_timer()
    }
}
