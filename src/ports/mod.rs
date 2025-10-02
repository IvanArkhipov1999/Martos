use core::time::Duration;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
#[cfg(feature = "network")]
use esp_wifi::esp_now::EspNow;

/// PortTrait contains all the platform specific functions.
pub trait PortTrait {
    /// Function is called when timer is created. Can be used to set configuration.
    fn setup_hardware_timer();
    /// Function is used to check the correctness of index.
    fn valid_timer_index(timer_index: u8) -> bool;
    /// Function is called to attempt to acquire the timer.
    fn try_acquire_timer(timer_index: u8) -> bool;
    /// Function is called to start the timer.
    fn start_hardware_timer(timer_index: u8);
    /// Function is called to change the timer operating mode.
    fn set_reload_mode(timer_index: u8, auto_reload: bool);
    /// Function is called to change the period of the timer.
    fn change_period_timer(timer_index: u8, period: Duration);
    /// Function is called to get amount of time from the start of the timer.
    fn get_time(timer_index: u8) -> Duration;
    /// Function is called to stop the timer.
    fn stop_hardware_timer(timer_index: u8) -> bool;
    /// Function is called to release the timer.
    fn release_hardware_timer(timer_index: u8);

    /// Function is called when heap is created. Can be used to set configuration.
    fn init_heap();
    #[cfg(feature = "network")]
    /// Function for initializing network settings.
    fn init_network();
    #[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
    #[cfg(feature = "network")]
    /// Function for getting esp-now object for network.
    fn get_esp_now() -> EspNow<'static>;
    #[cfg(feature = "uart")]
    type Uart2Type;
    #[cfg(feature = "uart")]
    type IoType;
    /// Initializes the UART subsystem for the current platform.
    ///
    /// This function performs platform-specific initialization of UART peripherals,
    /// including clock setup and basic port configuration.
    ///
    /// # Safety
    ///
    /// This function should be called only once during system initialization.
    /// Multiple calls may result in undefined behavior.
    ///
    /// # Availability
    ///
    /// Available only on ESP32 architectures (xtensa, riscv32) and only when
    /// the "uart" feature is enabled.
    #[cfg(feature = "uart")]
    fn setup_uart();
    /// Returns a UART2 peripheral instance for configuration and use.
    ///
    /// This function transfers ownership of the UART2 peripheral from the system
    /// pool to user code. After calling this function, attempting to retrieve
    /// the same instance again will fail until system restart.
    ///
    /// # Returns
    ///
    /// Returns `esp_hal::peripherals::UART2` - a UART2 peripheral instance
    /// ready for initialization with `esp_hal::uart::Uart::new()`.
    ///
    /// # Panics
    ///
    /// This function panics if:
    /// - UART2 has already been retrieved previously
    /// - The system was not initialized via `setup_uart()`
    ///
    /// # Availability
    ///
    /// Available only on ESP32 architectures (xtensa, riscv32) and only when
    /// the "uart" feature is enabled.
    #[cfg(feature = "uart")]
    fn get_uart2() -> Self::Uart2Type;
    /// Returns a GPIO/IO peripheral instance for pin configuration.
    ///
    /// This function transfers ownership of the GPIO peripheral from the system
    /// pool to user code for configuring input/output pins. After calling this
    /// function, attempting to retrieve the same instance again will fail.
    ///
    /// # Returns
    ///
    /// Returns `esp_hal::gpio::Io` - a GPIO peripheral instance that provides
    /// access to individual pins through the `pins` field.
    ///
    /// # Panics
    ///
    /// This function panics if:
    /// - The IO peripheral has already been retrieved previously
    /// - The system was not initialized via `setup_uart()`
    ///
    /// # Availability
    ///
    /// Available only on ESP32 architectures (xtensa, riscv32) and only when
    /// the "uart" feature is enabled.
    #[cfg(feature = "uart")]
    fn get_io() -> Self::IoType;

    // TODO: split to separate trait?
    #[cfg(feature = "preemptive")]
    fn setup_interrupt() {}
    #[cfg(feature = "preemptive")]
    #[allow(private_interfaces)]
    fn setup_stack(_thread: &mut crate::task_manager::preemptive::Thread) {}
    #[cfg(feature = "preemptive")]
    fn save_ctx(_thread_ctx: &mut TrapFrame, _isr_ctx: &TrapFrame) {}
    #[cfg(feature = "preemptive")]
    fn load_ctx(_thread_ctx: &TrapFrame, _isr_ctx: &mut TrapFrame) {}
}

/// Port is an alias of PortTrait implementation for a current platform

#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub mod xtensa_esp32;
#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
mod arch {
    use super::xtensa_esp32;
    pub type Port = xtensa_esp32::XtensaEsp32;
    #[cfg(feature = "preemptive")]
    pub type TrapFrame = xtensa_esp32::TrapFrame;
    #[cfg(feature = "preemptive")]
    pub const STACK_ALIGN: usize = 16;
}

#[cfg(all(
    not(any(target_arch = "riscv32", target_arch = "xtensa")),
    not(target_arch = "mips64")
))]
pub mod mok;
#[cfg(all(
    not(any(target_arch = "riscv32", target_arch = "xtensa")),
    not(target_arch = "mips64")
))]
mod arch {
    use super::mok;
    pub type Port = mok::Mok;
    #[cfg(feature = "preemptive")]
    pub type TrapFrame = mok::TrapFrame;
    #[cfg(feature = "preemptive")]
    pub const STACK_ALIGN: usize = 0;
}

#[cfg(any(target_arch = "mips64", feature = "mips64_timer_tests"))]
pub mod mips64;
#[cfg(target_arch = "mips64")]
mod arch {
    use super::mips64;
    pub type Port = mips64::Mips64;
    #[cfg(feature = "preemptive")]
    pub type TrapFrame = ();
    #[cfg(feature = "preemptive")]
    pub const STACK_ALIGN: usize = 0;
}

pub use arch::*;

#[cfg(feature = "uart")]
pub type Uart2Type = <Port as PortTrait>::Uart2Type;
#[cfg(feature = "uart")]
pub type IoType = <Port as PortTrait>::IoType;
