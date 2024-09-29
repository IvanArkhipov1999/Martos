pub mod hardware_timer;
pub mod memory_manager;
pub mod network;
use crate::ports::PortTrait;

/// PortTrait implementation for XtensaEsp32 platform
pub struct XtensaEsp32;
impl PortTrait for XtensaEsp32 {
    fn init_heap() {
        memory_manager::init_heap();
    }

    fn setup_hardware_timer() {
        hardware_timer::setup_hardware_timer();
    }

    fn get_tick_counter() -> crate::timer::TickType {
        hardware_timer::get_tick_counter()
    }

    fn init_network() {
        network::init_network();
    }
}
