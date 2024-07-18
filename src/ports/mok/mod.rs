pub mod hardware_timer;
pub mod memory_manager;
use crate::ports::Port;

pub struct Mok;
impl Port for Mok{
    fn init_heap() {
        memory_manager::init_heap();
    }

    fn setup_hardware_timer(){
        hardware_timer::setup_hardware_timer();
    }

    fn get_tick_counter() -> crate::timer::TickType {
        hardware_timer::get_tick_counter()
    }
}
