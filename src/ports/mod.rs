use crate::timer::TickType;

pub trait Port{
    fn setup_hardware_timer();
    fn get_tick_counter() -> TickType;

    fn init_heap();
}


#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
pub mod mok;
#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
pub type PORT = mok::Mok;

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub mod xtensa_esp32;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub type PORT = xtensa_esp32::XtensaEsp32;
