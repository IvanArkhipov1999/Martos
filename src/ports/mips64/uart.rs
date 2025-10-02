//! MIPS64 UART stub implementation

/// MIPS64 UART2 peripheral stub
#[cfg(feature = "uart")]
pub struct Mips64Uart2;

/// MIPS64 IO peripheral stub
#[cfg(feature = "uart")]
pub struct Mips64Io;

/// Initialize UART subsystem (stub implementation)
#[cfg(feature = "uart")]
pub fn setup_uart() {
    // MIPS64 UART not implemented yet
    // Could implement real MIPS64 UART initialization here in the future
}

/// Get MIPS64 UART2 peripheral instance
#[cfg(feature = "uart")]
pub fn get_uart2() -> Mips64Uart2 {
    Mips64Uart2
}

/// Get MIPS64 IO peripheral instance
#[cfg(feature = "uart")]
pub fn get_io() -> Mips64Io {
    Mips64Io
}

/// Type alias for consistency with PortTrait
#[cfg(feature = "uart")]
pub type Uart2Type = Mips64Uart2;

/// Type alias for consistency with PortTrait
#[cfg(feature = "uart")]
pub type IoType = Mips64Io;
