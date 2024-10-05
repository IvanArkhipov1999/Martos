pub mod hardware_timer;
pub mod memory_manager;
#[cfg(feature = "network")]
pub mod network;
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

    fn get_tick_counter() -> crate::timer::TickType {
        hardware_timer::get_tick_counter()
    }

    #[cfg(feature = "network")]
    fn init_network() {
        network::init_network();
    }
}
