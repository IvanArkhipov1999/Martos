pub mod hardware_timer;
pub mod memory_manager;
#[cfg(feature = "network")]
pub mod network;
#[cfg(feature = "uart")]
pub mod uart;

use crate::ports::PortTrait;

/// PortTrait implementation for Mok platform
pub struct Mok;
impl PortTrait for Mok {
    #[cfg(feature = "uart")]
    type Uart2Type = uart::Uart2Type;
    
    #[cfg(feature = "uart")]
    type IoType = uart::IoType;

    fn init_heap() {
        memory_manager::init_heap();
    }

    fn setup_hardware_timer() {
        hardware_timer::setup_hardware_timer();
    }

    fn valid_timer_index(_timer_index: u8) -> bool {
        true
    }

    fn try_acquire_timer(_timer_index: u8) -> bool {
        true
    }

    fn start_hardware_timer(_timer_index: u8) {
        hardware_timer::start_hardware_timer();
    }

    fn set_reload_mode(_timer_index: u8, auto_reload: bool) {
        hardware_timer::set_reload_mode(auto_reload);
    }

    fn change_period_timer(_timer_index: u8, period: core::time::Duration) {
        hardware_timer::change_period_timer(period);
    }

    fn get_time(_timer_index: u8) -> core::time::Duration {
        hardware_timer::get_time()
    }

    fn stop_hardware_timer(_timer_index: u8) -> bool {
        false
    }

    fn release_hardware_timer(_timer_index: u8) {
        hardware_timer::release_hardware_timer()
    }

    #[cfg(feature = "network")]
    fn init_network() {
        network::init_network();
    }
    #[cfg(feature = "preemptive")]
    fn setup_interrupt() {}
    #[cfg(feature = "preemptive")]
    fn setup_stack(thread: &mut crate::task_manager::preemptive::Thread) {}
    #[cfg(feature = "preemptive")]
    fn save_ctx(thread_ctx: &mut crate::ports::TrapFrame, isr_ctx: &crate::ports::TrapFrame) {}
    #[cfg(feature = "preemptive")]
    fn load_ctx(thread_ctx: &crate::ports::TrapFrame, isr_ctx: &mut crate::ports::TrapFrame) {}

    #[cfg(feature = "uart")]
    fn setup_uart() {
        uart::setup_uart();
    }

    #[cfg(feature = "uart")]
    fn get_uart2() -> Self::Uart2Type {
        uart::get_uart2()
    }

    #[cfg(feature = "uart")]
    fn get_io() -> Self::IoType {
        uart::get_io()
    }
}

#[allow(dead_code)]
pub type TrapFrame = ();
