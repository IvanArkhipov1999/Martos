pub mod hardware_timer;
pub mod memory_manager;
#[cfg(feature = "network")]
pub mod network;
#[cfg(feature = "preemptive")]
mod preempt;

use crate::ports::PortTrait;
#[cfg(feature = "network")]
use esp_wifi::esp_now::EspNow;

// TODO: make it port just for esp32, not only for XtensaEsp32
/// PortTrait implementation for XtensaEsp32 platform
pub struct XtensaEsp32;
impl PortTrait for XtensaEsp32 {
    fn setup_hardware_timer() {
        hardware_timer::setup_hardware_timer();
    }

    fn get_tick_counter() -> crate::timer::TickType {
        hardware_timer::get_tick_counter()
    }

    fn init_heap() {
        memory_manager::init_heap();
    }

    #[cfg(feature = "network")]
    fn init_network() {
        network::init_network();
    }

    #[cfg(feature = "network")]
    fn get_esp_now() -> EspNow<'static> {
        network::get_esp_now()
    }

    #[cfg(feature = "preemptive")]
    fn setup_interrupt() {
        preempt::setup_interrupt();
    }
    #[cfg(feature = "preemptive")]
    fn setup_stack(thread: &mut crate::task_manager::tm::Thread) {
        preempt::setup_stack(thread);
    }
    #[cfg(feature = "preemptive")]
    fn save_ctx(thread_ctx: &mut TrapFrame, isr_ctx: &TrapFrame) {
        preempt::save_ctx(thread_ctx, isr_ctx)
    }
    #[cfg(feature = "preemptive")]
    fn load_ctx(thread_ctx: &TrapFrame, isr_ctx: &mut TrapFrame) {
        preempt::load_ctx(thread_ctx, isr_ctx)
    }
}
pub type TrapFrame = esp_hal::trapframe::TrapFrame;
