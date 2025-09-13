//! Mock UART implementation for testing and simulation

/// Mock UART2 peripheral for platforms that don't support real UART
#[cfg(feature = "uart")]
pub struct MockUart2;

/// Mock IO peripheral for platforms that don't support real UART
#[cfg(feature = "uart")]
pub struct MockIo;

/// Initialize UART subsystem (mock implementation)
#[cfg(feature = "uart")]
pub fn setup_uart() {
    // Mock implementation - does nothing
    // Could log or simulate UART initialization here
}

/// Get mock UART2 peripheral instance
#[cfg(feature = "uart")]
pub fn get_uart2() -> MockUart2 {
    MockUart2
}

/// Get mock IO peripheral instance
#[cfg(feature = "uart")]
pub fn get_io() -> MockIo {
    MockIo
}

/// Type alias for consistency with PortTrait
#[cfg(feature = "uart")]
pub type Uart2Type = MockUart2;

/// Type alias for consistency with PortTrait
#[cfg(feature = "uart")]
pub type IoType = MockIo;
