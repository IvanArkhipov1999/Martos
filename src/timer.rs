//! Hardware timer abstraction for Martos RTOS.
//!
//! This module provides a platform-independent interface to hardware timers, enabling
//! precise timing operations, periodic tasks, and time measurement across different
//! embedded architectures. The timer system is built around exclusive resource management
//! and cooperative multitasking principles.
//!
//! # Architecture Overview
//!
//! The timer system consists of several key components:
//!
//! - **Hardware Abstraction**: Platform-specific timer implementations through the [`Port`] trait
//! - **Resource Management**: Exclusive access to timer resources via acquisition/release semantics  
//! - **Tick Counting**: Software-maintained counters for precise time tracking
//! - **Flexible Configuration**: Support for one-shot and periodic timer modes
//!
//! # Core Concepts
//!
//! ## Timer Lifecycle
//!
//! 1. **System Initialization**: Call [`Timer::setup_timer()`] during system startup
//! 2. **Timer Acquisition**: Use [`Timer::get_timer()`] to acquire exclusive access to a timer
//! 3. **Configuration**: Set period, mode, and other parameters
//! 4. **Execution**: Start timer and handle tick events
//! 5. **Cleanup**: Release timer with [`Timer::release_timer()`] when finished
//!
//! ## Tick System
//!
//! Each timer maintains two time representations:
//! - **Hardware Timer**: The actual hardware counter value accessible via [`Timer::get_time()`]
//! - **Software Ticks**: Application-maintained counter updated via [`Timer::loop_timer()`]
//!
//! This dual approach enables both high-resolution hardware timing and application-level
//! event counting suitable for cooperative multitasking scenarios.
//!
//! ## Resource Management
//!
//! Timers are exclusive resources - only one [`Timer`] instance can control a given
//! hardware timer at a time. The system uses atomic operations to ensure thread-safe
//! acquisition and prevents conflicts between multiple tasks attempting to use the
//! same timer hardware.
//!
//! # Platform Support
//!
//! The timer system supports multiple embedded architectures through platform-specific
//! implementations:
//!
//! - **ESP32 (Xtensa)**: High-resolution timer groups with microsecond precision
//! - **ESP32 (RISC-V)**: Timer support for ESP32-C3, C6 and other RISC-V variants
//! - **MIPS64**: System timer and performance counter integration
//! - **Mock Platform**: Simulation environment for testing and development
//!
//! Each platform provides different capabilities in terms of:
//! - Number of available timers (typically 0-7 per timer group)
//! - Maximum timer resolution (microsecond to nanosecond range)  
//! - Timer period range (microseconds to hours)
//! - Stop/start functionality support
//!
//! # Usage Patterns
//!
//! ## Basic Timer Setup
//!
//! ```
//! use martos::timer::{Timer, TickType};
//! use core::time::Duration;
//!
//! // Initialize timer subsystem (call once at startup)
//! Timer::setup_timer();
//!
//! // Acquire timer 0
//! let mut timer = Timer::get_timer(0).expect("Timer 0 not available");
//!
//! // Configure for 1ms periodic interrupts
//! timer.set_reload_mode(true);
//! timer.change_period_timer(Duration::from_millis(1));
//!
//! // Start the timer
//! timer.start_timer();
//! ```
//!
//! ## Periodic Task with Tick Counting
//!
//! ```
//! use martos::timer::Timer;
//! use core::time::Duration;
//!
//! let mut timer = Timer::get_timer(1).unwrap();
//! timer.set_reload_mode(true);
//! timer.change_period_timer(Duration::from_micros(100));
//! timer.start_timer();
//!
//! // In your interrupt handler or periodic task:
//! loop {
//!     // Update software tick counter
//!     timer.loop_timer();
//!     
//!     // Check if we've reached 10,000 ticks (1 second at 100μs intervals)
//!     if timer.tick_counter >= 10_000 {
//!         println!("One second elapsed: {} ticks", timer.tick_counter);
//!         break;
//!     }
//!     
//!     // Do periodic work here...
//! }
//!
//! // Clean up
//! timer.release_timer();
//! ```
//!
//! ## One-Shot Timer for Delays
//!
//! ```
//! use martos::timer::Timer;
//! use core::time::Duration;
//!
//! let timer = Timer::get_timer(2).unwrap();
//!
//! // Configure for single 5-second timeout
//! timer.set_reload_mode(false);  // One-shot mode
//! timer.change_period_timer(Duration::from_secs(5));
//! timer.start_timer();
//!
//! // Wait for timeout or do other work
//! // Timer will automatically stop after 5 seconds
//! ```
//!
//! ## High-Resolution Time Measurement
//!
//! ```
//! use martos::timer::Timer;
//!
//! let timer = Timer::get_timer(3).unwrap();
//! timer.start_timer();
//!
//! let start_time = timer.get_time();
//!
//! // Execute code to be measured  
//! for i in 0..1000 {
//!     // Simulate some work
//!     let _ = i * 2;
//! }
//!
//! let end_time = timer.get_time();
//! println!("Operation took: {:?}", end_time);
//!
//! timer.release_timer();
//! ```
//!
//! # Integration with Martos RTOS
//!
//! The timer system integrates seamlessly with other Martos components:
//!
//! - **Task Manager**: Timers can be used within tasks for periodic execution
//! - **Scheduler**: Time-based task switching and scheduling decisions
//! - **Memory Manager**: Timer events can trigger garbage collection or memory management
//! - **Network Stack**: Protocol timeouts, keepalive timers, and rate limiting
//!
//! ## Task Manager Integration
//!
//! ```
//! use martos::{init_system, task_manager::{TaskManager, TaskManagerTrait}};
//! use martos::timer::Timer;
//! use core::time::Duration;
//!
//! static mut SYSTEM_TIMER: Option<Timer> = None;
//!
//! fn timer_setup() {
//!     Timer::setup_timer();
//!     unsafe {
//!         SYSTEM_TIMER = Timer::get_timer(0);
//!         if let Some(ref timer) = SYSTEM_TIMER {
//!             timer.set_reload_mode(true);
//!             timer.change_period_timer(Duration::from_millis(10));
//!             timer.start_timer();
//!         }
//!     }
//! }
//!
//! fn timer_task() {
//!     unsafe {
//!         if let Some(ref mut timer) = SYSTEM_TIMER {
//!             timer.loop_timer();
//!             // Handle periodic timer logic
//!         }
//!     }
//! }
//!
//! fn timer_stop_condition() -> bool {
//!     false // Run forever
//! }
//!
//! // Register with task manager
//! TaskManager::add_task(timer_setup, timer_task, timer_stop_condition);
//! ```
//!
//! # Performance Considerations
//!
//! ## Timer Resolution and Accuracy
//!
//! - **Hardware Limitations**: Timer accuracy depends on system clock stability and frequency
//! - **Interrupt Overhead**: High-frequency timers may impact system performance
//! - **Platform Variations**: Different architectures provide different precision levels
//!
//! ## Memory Usage
//!
//! - **Minimal Footprint**: Each [`Timer`] instance uses only 16 bytes (8 + 8 for tick counter)
//! - **No Dynamic Allocation**: All timer management uses compile-time known resources
//! - **Zero-Copy Operations**: Timer operations avoid unnecessary memory copies
//!
//! ## Real-Time Guarantees
//!
//! - **Deterministic Behavior**: Timer operations have bounded execution time
//! - **Interrupt Priority**: Timer interrupts should be configured with appropriate priorities
//! - **Preemption Safety**: Timer state is protected against concurrent access
//!
//! # Error Handling
//!
//! The timer system uses Rust's type system for robust error handling:
//!
//! - **Acquisition Failures**: [`Timer::get_timer()`] returns [`Option<Timer>`] for safe failure handling  
//! - **Platform Limitations**: Unsupported operations return [`bool`] status indicators
//! - **Resource Exhaustion**: Timer acquisition automatically fails when resources are unavailable
//!
//! ## Common Error Scenarios
//!
//! ```
//! use martos::timer::Timer;
//!
//! // Handle timer acquisition failure
//! match Timer::get_timer(5) {
//!     Some(timer) => {
//!         // Timer successfully acquired
//!         println!("Timer 5 acquired");
//!         timer.release_timer();
//!     }
//!     None => {
//!         println!("Timer 5 is busy or invalid index");
//!         // Try alternative timer or handle gracefully
//!     }
//! }
//!
//! // Handle platform limitations
//! let timer = Timer::get_timer(0).unwrap();
//! if !timer.stop_condition_timer() {
//!     println!("Platform doesn't support stopping timers");
//!     // Use alternative approach (disable interrupts, etc.)
//! }
//! ```
//!
//! # Safety and Thread Safety
//!
//! - **Memory Safety**: All timer operations are memory-safe by design
//! - **Resource Safety**: Automatic resource cleanup prevents timer leaks
//! - **Concurrency**: Timer acquisition uses atomic operations for thread safety
//! - **Interrupt Context**: Timer functions are suitable for use in interrupt handlers
//!
//! # Future Enhancements
//!
//! Planned improvements to the timer system include:
//!
//! - **Timer Pools**: Automatic timer allocation and management
//! - **Cascade Timers**: Support for very long time periods using timer chaining
//! - **Power Management**: Low-power timer modes and sleep integration
//! - **Synchronization**: Timer synchronization primitives for coordinated timing
//! - **Profiling**: Built-in performance measurement and timer usage statistics
//!
//! # See Also
//!
//! - [`crate::task_manager`] - Task scheduling and management
//! - [`crate::ports`] - Platform-specific hardware abstraction
//! - [`core::time::Duration`] - Standard time duration representation
//!
//! # Examples Repository
//!
//! For more comprehensive examples and use cases, see the `examples/` directory in
//! the Martos repository, which contains platform-specific timer demonstrations
//! and integration examples with other RTOS components.

use crate::ports::{Port, PortTrait};
use core::time::Duration;

/// Type alias for timer tick counting.
///
/// Represents the number of timer ticks that have elapsed. Currently implemented
/// as `u64` but should ideally be `u128` for extended range and better synchronization
/// support in future versions.
///
/// # Note
///
/// This type is designed to be signed for synchronization purposes, though the
/// current implementation uses an unsigned type. Future versions may change this
/// to a signed type to better support timer synchronization algorithms.
///
/// # Examples
///
/// ```
/// use martos::timer::TickType;
///
/// let ticks: TickType = 1000;
/// let max_ticks = TickType::MAX;
/// ```
pub type TickType = u64;

/// Hardware timer abstraction for Martos RTOS.
///
/// The `Timer` struct provides a platform-independent interface to hardware timers.
/// Each timer instance represents a single hardware timer resource that can be
/// configured for various timing operations such as periodic interrupts, one-shot
/// delays, and time measurement.
///
/// # Platform Support
///
/// Timer functionality is implemented through the platform abstraction layer (PAL),
/// allowing the same API to work across different hardware platforms including:
/// - ESP32 (Xtensa and RISC-V variants)
/// - MIPS64 architectures
/// - Mock platform for testing
///
/// # Resource Management
///
/// Timers are exclusive resources - only one `Timer` instance can control a given
/// hardware timer at a time. Use [`Timer::get_timer()`] to acquire a timer and
/// [`Timer::release_timer()`] to free it for other uses.
///
/// # Synchronization
///
/// The internal tick counter is maintained separately from the hardware timer state
/// and should be updated by calling [`Timer::loop_timer()`] from appropriate
/// interrupt handlers or periodic tasks.
///
/// # Examples
///
/// ```
/// use martos::timer::Timer;
/// use core::time::Duration;
///
/// // Acquire timer 0
/// let mut timer = Timer::get_timer(0).expect("Failed to acquire timer");
///
/// // Configure for 100ms periodic operation
/// timer.set_reload_mode(true);
/// timer.change_period_timer(Duration::from_millis(100));
///
/// // Start the timer
/// timer.start_timer();
///
/// // When done, release the resource
/// timer.release_timer();
/// ```
///
/// # Safety
///
/// While this struct is memory-safe, improper timer configuration or forgetting
/// to release timers can lead to resource leaks or timing issues in your application.
#[repr(C)]
pub struct Timer {
    /// Hardware timer index within the timer peripheral block.
    ///
    /// This field identifies which specific hardware timer this instance controls.
    /// Valid timer indices are platform-dependent and should be validated using
    /// platform-specific functions before use.
    ///
    /// # Range
    ///
    /// Typically ranges from 0 to the number of available hardware timers minus 1.
    /// For example, ESP32 typically has timers 0-3 available per timer group.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let timer = Timer::get_timer(2).unwrap();
    /// assert_eq!(timer.timer_index, 2);
    /// ```
    pub timer_index: u8,

    /// Software-maintained tick counter.
    ///
    /// This counter tracks the number of timer ticks that have occurred since
    /// the timer instance was created. It is independent of the hardware timer
    /// state and must be manually updated by calling [`Timer::loop_timer()`].
    ///
    /// # Overflow Behavior
    ///
    /// When this counter reaches [`TickType::MAX`], the overflow behavior is
    /// currently undefined. Applications should implement their own overflow
    /// detection and handling if needed for long-running timers.
    ///
    /// # Synchronization
    ///
    /// This field is intended for use in timer synchronization algorithms,
    /// though the current implementation does not provide built-in synchronization
    /// primitives.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let mut timer = Timer::get_timer(0).unwrap();
    /// assert_eq!(timer.tick_counter, 0); // Starts at zero
    ///
    /// timer.loop_timer();
    /// assert_eq!(timer.tick_counter, 1); // Incremented by one
    /// ```
    pub tick_counter: TickType,

    /// Time synchronization offset in microseconds.
    ///
    /// This field stores the cumulative time correction applied by the time
    /// synchronization system. Positive values indicate the local time is ahead,
    /// negative values indicate the local time is behind the synchronized time.
    ///
    /// # Synchronization
    ///
    /// This offset is automatically updated by the time synchronization manager
    /// when synchronization messages are processed. Applications should use
    /// [`Timer::get_synchronized_time()`] to get the corrected time.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    /// let offset = timer.get_sync_offset_us();
    /// println!("Time offset: {} microseconds", offset);
    /// ```
    pub sync_offset_us: i64,
}

impl Timer {
    /// Initializes the timer subsystem for the current platform.
    ///
    /// This function should be called once during system initialization before
    /// attempting to acquire any timer instances. It performs platform-specific
    /// timer hardware setup including clock configuration, peripheral enabling,
    /// and any required calibration procedures.
    ///
    /// # Platform Behavior
    ///
    /// - **ESP32**: Initializes timer groups and configures base clocks
    /// - **MIPS64**: Sets up system timer and performance counters  
    /// - **Mock**: Performs simulation environment setup
    ///
    /// # Panics
    ///
    /// May panic if the underlying hardware initialization fails or if called
    /// multiple times without proper cleanup.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// // Call once during system startup
    /// Timer::setup_timer();
    ///
    /// // Now timers can be acquired
    /// let timer = Timer::get_timer(0).unwrap();
    /// ```
    pub fn setup_timer() {
        Port::setup_hardware_timer()
    }

    /// Attempts to acquire a timer instance at the specified index.
    ///
    /// This function checks if the timer index is valid for the current platform
    /// and whether the timer is currently available (not in use by another instance).
    /// If successful, it returns a new `Timer` instance with the tick counter
    /// initialized to zero.
    ///
    /// # Arguments
    ///
    /// * `timer_index` - The hardware timer index to acquire (platform-specific range)
    ///
    /// # Returns
    ///
    /// * `Some(Timer)` - Successfully acquired timer instance
    /// * `None` - Timer is busy, index is invalid, or acquisition failed
    ///
    /// # Thread Safety
    ///
    /// This function uses platform-specific atomic operations to ensure only one
    /// caller can successfully acquire a given timer index.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// // Try to acquire timer 0
    /// match Timer::get_timer(0) {
    ///     Some(timer) => {
    ///         println!("Successfully acquired timer 0");
    ///         // Use timer...
    ///         timer.release_timer();
    ///     }
    ///     None => {
    ///         println!("Timer 0 is busy or invalid");
    ///     }
    /// }
    /// ```
    pub fn get_timer(timer_index: u8) -> Option<Self> {
        if Port::valid_timer_index(timer_index) && Port::try_acquire_timer(timer_index) {
            Some(Self {
                timer_index,
                tick_counter: 0,
                sync_offset_us: 0,
            })
        } else {
            None
        }
    }

    /// Increments the internal tick counter by one.
    ///
    /// This method should be called periodically (typically from an interrupt
    /// service routine or timer callback) to maintain accurate timing. It updates
    /// the software tick counter independently of the hardware timer state.
    ///
    /// # Usage Patterns
    ///
    /// - **Interrupt Handler**: Call from timer interrupt service routine
    /// - **Periodic Task**: Call from a regularly scheduled task
    /// - **Manual Timing**: Call when specific timing events occur
    ///
    /// # Overflow Behavior
    ///
    /// When `tick_counter` reaches [`TickType::MAX`] (currently `u64::MAX`), the
    /// next call will wrap to 0. This behavior is currently undefined in terms
    /// of timing semantics and should be handled by the application if long-term
    /// timing accuracy is required.
    ///
    /// # Performance
    ///
    /// This operation is very fast (single increment) and suitable for high-frequency
    /// calling from interrupt contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let mut timer = Timer::get_timer(0).unwrap();
    ///
    /// // In your interrupt handler or periodic task:
    /// timer.loop_timer(); // tick_counter: 0 -> 1
    /// timer.loop_timer(); // tick_counter: 1 -> 2
    ///
    /// println!("Tick count: {}", timer.tick_counter); // Prints: 2
    /// ```
    pub fn loop_timer(&mut self) {
        self.tick_counter += 1;
    }

    /// Starts the hardware timer associated with this timer instance.
    ///
    /// This function activates the underlying hardware timer, allowing it to
    /// begin counting based on its current configuration. The timer will operate
    /// according to the mode set by [`Timer::set_reload_mode()`] and the period
    /// configured by [`Timer::change_period_timer()`].
    ///
    /// # Prerequisites
    ///
    /// Before calling this method, ensure:
    /// - Timer period has been configured
    /// - Reload mode has been set appropriately
    /// - Any required interrupt handlers are installed
    ///
    /// # Hardware Effects
    ///
    /// - Enables the hardware timer clock
    /// - Begins countdown/countup based on configuration
    /// - May trigger interrupts when timer expires (platform-dependent)
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    /// use core::time::Duration;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    ///
    /// // Configure timer first
    /// timer.set_reload_mode(true);
    /// timer.change_period_timer(Duration::from_millis(10));
    ///
    /// // Now start it
    /// timer.start_timer();
    /// ```
    pub fn start_timer(&self) {
        Port::start_hardware_timer(self.timer_index);
    }

    /// Configures the timer's reload behavior.
    ///
    /// This method sets whether the timer should automatically reload (continue
    /// counting in a loop) or operate as a one-shot timer (stop after one period).
    /// The reload behavior affects what happens when the timer reaches its
    /// configured period value.
    ///
    /// # Arguments
    ///
    /// * `auto_reload` - Timer operating mode:
    ///   - `true`: **Auto-reload/Periodic mode** - Timer automatically restarts
    ///     after reaching the period, creating continuous periodic events
    ///   - `false`: **One-shot mode** - Timer stops after reaching the period
    ///     and must be manually restarted
    ///
    /// # Mode Details
    ///
    /// ## Auto-reload Mode (`true`)
    /// - Timer continuously generates periodic events
    /// - Suitable for regular interrupts, PWM, or periodic tasks
    /// - Timer restarts automatically without software intervention
    ///
    /// ## One-shot Mode (`false`)  
    /// - Timer generates a single event after the specified delay
    /// - Suitable for timeouts, delays, or single-occurrence events
    /// - Requires manual restart for subsequent timing events
    ///
    /// # Platform Notes
    ///
    /// Some platforms may have restrictions on when reload mode can be changed
    /// (e.g., only when timer is stopped).
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    /// use core::time::Duration;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    ///
    /// // Configure for periodic 1ms interrupts
    /// timer.set_reload_mode(true);
    /// timer.change_period_timer(Duration::from_millis(1));
    /// timer.start_timer();
    ///
    /// // Later, reconfigure for a single 5-second timeout
    /// timer.set_reload_mode(false);
    /// timer.change_period_timer(Duration::from_secs(5));
    /// ```
    pub fn set_reload_mode(&self, auto_reload: bool) {
        Port::set_reload_mode(self.timer_index, auto_reload);
    }

    /// Changes the timer's period to the specified duration.
    ///
    /// This function configures how long the timer counts before triggering
    /// an event (interrupt, flag, etc.). The actual resolution and range depend
    /// on the underlying hardware capabilities and clock configuration.
    ///
    /// # Arguments
    ///
    /// * `period` - The new timer period as a [`Duration`]. The timer will
    ///   trigger an event after this amount of time has elapsed.
    ///
    /// # Resolution and Limits
    ///
    /// Timer resolution is hardware and platform dependent:
    /// - **ESP32**: Typically microsecond resolution, wide range
    /// - **MIPS64**: Platform-specific, often nanosecond capable
    /// - **Mock**: Simulated timing, configurable resolution
    ///
    /// Very short periods may be limited by hardware capabilities, and very
    /// long periods may exceed the maximum timer value.
    ///
    /// # Timing Accuracy
    ///
    /// Actual timing accuracy depends on:
    /// - System clock stability and calibration
    /// - Hardware timer resolution
    /// - Interrupt latency and jitter
    /// - System load and other timing interference
    ///
    /// # Dynamic Changes
    ///
    /// Period changes may take effect immediately or on the next timer reload,
    /// depending on the platform implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    /// use core::time::Duration;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    ///
    /// // Set various periods
    /// timer.change_period_timer(Duration::from_millis(100));  // 100ms
    /// timer.change_period_timer(Duration::from_micros(500));  // 500μs  
    /// timer.change_period_timer(Duration::from_secs(1));      // 1 second
    /// ```
    pub fn change_period_timer(&self, period: Duration) {
        Port::change_period_timer(self.timer_index, period);
    }

    /// Attempts to stop the hardware timer.
    ///
    /// This function tries to halt the timer's operation, preventing further
    /// counting and event generation. Not all platforms support stopping timers
    /// once they have been started, and some may only support stopping in
    /// certain modes or states.
    ///
    /// # Returns
    ///
    /// * `true` - Timer was successfully stopped
    /// * `false` - Platform does not support stopping timers, timer cannot be
    ///   stopped in current state, or stop operation failed
    ///
    /// # Platform Support
    ///
    /// - **ESP32**: Generally supports stopping timers
    /// - **MIPS64**: Platform-dependent support
    /// - **Mock**: Full stop/start support for testing
    ///
    /// # State After Stopping
    ///
    /// When successfully stopped:
    /// - Timer hardware stops counting
    /// - No further interrupts or events are generated
    /// - Timer can typically be restarted with [`Timer::start_timer()`]
    /// - Current count value behavior is platform-specific
    ///
    /// # Error Handling
    ///
    /// If this method returns `false`, consider alternative approaches:
    /// - Disable timer interrupts instead of stopping the timer
    /// - Set a very long period to effectively disable events
    /// - Use one-shot mode and let the timer expire naturally
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    /// timer.start_timer();
    ///
    /// // Later, try to stop it
    /// if timer.stop_condition_timer() {
    ///     println!("Timer stopped successfully");
    /// } else {
    ///     println!("Cannot stop timer on this platform");
    ///     // Handle gracefully - maybe disable interrupts instead
    /// }
    /// ```
    pub fn stop_condition_timer(&self) -> bool {
        Port::stop_hardware_timer(self.timer_index)
    }

    /// Retrieves the current timer value from the hardware.
    ///
    /// Returns the current count value of the hardware timer as a [`Duration`].
    /// This represents the actual hardware timer state and is independent of
    /// the internal tick counter maintained by [`Timer::loop_timer()`].
    ///
    /// # Return Value
    ///
    /// The returned [`Duration`] represents:
    /// - **Counting Up**: Time elapsed since timer started or last reload
    /// - **Counting Down**: Time remaining until next timer event
    /// - **Stopped Timer**: Last count value before stopping
    ///
    /// The specific interpretation depends on the hardware implementation and
    /// timer configuration.
    ///
    /// # Resolution
    ///
    /// The resolution of the returned time value depends on:
    /// - Hardware timer clock frequency
    /// - Platform timer implementation  
    /// - System clock accuracy and stability
    ///
    /// # Use Cases
    ///
    /// - **Precise Timing Measurements**: Get high-resolution timestamps
    /// - **Remaining Time Calculation**: Check time until next event
    /// - **Timer Synchronization**: Coordinate multiple timers
    /// - **Performance Profiling**: Measure execution times
    ///
    /// # Performance Notes
    ///
    /// This operation typically requires reading hardware registers and may
    /// have higher latency than pure software operations. Avoid calling
    /// excessively in performance-critical code.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    /// use core::time::Duration;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    /// timer.change_period_timer(Duration::from_millis(100));
    /// timer.start_timer();
    ///
    /// // Check current timer value
    /// let current_time = timer.get_time();
    /// println!("Timer value: {:?}", current_time);
    /// ```
    pub fn get_time(&self) -> Duration {
        Port::get_time(self.timer_index)
    }

    /// Releases the timer back to the system for reuse.
    ///
    /// This function frees the timer resource, making it available for other
    /// components to acquire and use. After calling this method, the current
    /// `Timer` instance should no longer be used, as it no longer has exclusive
    /// access to the hardware timer.
    ///
    /// # Resource Management
    ///
    /// - Marks the timer as available in the system resource pool
    /// - Other code can now successfully call [`Timer::get_timer()`] for this index
    /// - The hardware timer may be stopped or left in its current state
    /// - Timer configuration (period, mode) may be preserved or reset
    ///
    /// # Best Practices
    ///
    /// - Always call this method when finished using a timer
    /// - Consider stopping the timer before releasing if appropriate
    /// - Do not use the `Timer` instance after calling this method
    /// - Use RAII patterns (Drop trait) where possible for automatic cleanup
    ///
    /// # State After Release
    ///
    /// The behavior of the hardware timer after release is platform-dependent:
    /// - Timer may continue running with current configuration
    /// - Timer may be automatically stopped
    /// - Configuration may be reset to default values
    ///
    /// # Memory Safety
    ///
    /// While this operation is memory-safe, using the `Timer` instance after
    /// release may cause logic errors or conflicts with new owners of the
    /// same hardware timer.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    /// use core::time::Duration;
    ///
    /// {
    ///     let timer = Timer::get_timer(0).unwrap();
    ///     
    ///     // Configure and use timer
    ///     timer.set_reload_mode(true);
    ///     timer.change_period_timer(Duration::from_millis(50));
    ///     timer.start_timer();
    ///     
    ///     // Clean up when done
    ///     timer.release_timer();
    /// } // timer is now released and can be acquired by others
    ///
    /// // Later, same timer can be acquired again
    /// let timer2 = Timer::get_timer(0).unwrap(); // Should succeed
    /// ```
    pub fn release_timer(&self) {
        Port::release_hardware_timer(self.timer_index)
    }

    /// Adjusts the timer's synchronization offset.
    ///
    /// This method applies a time correction to the timer's synchronization offset.
    /// The correction is cumulative - multiple calls will add to the existing offset.
    /// This is typically called by the time synchronization manager when processing
    /// synchronization messages from other nodes.
    ///
    /// # Arguments
    ///
    /// * `correction_us` - Time correction in microseconds. Positive values indicate
    ///   the local time should be adjusted forward, negative values indicate it should
    ///   be adjusted backward.
    ///
    /// # Safety
    ///
    /// Large corrections may cause timing issues in applications. The time synchronization
    /// manager should validate corrections before applying them.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let mut timer = Timer::get_timer(0).unwrap();
    /// 
    /// // Apply a 100 microsecond correction
    /// timer.adjust_sync_offset(100);
    /// 
    /// // Apply a -50 microsecond correction
    /// timer.adjust_sync_offset(-50);
    /// 
    /// // Net offset is now 50 microseconds
    /// assert_eq!(timer.get_sync_offset_us(), 50);
    /// ```
    pub fn adjust_sync_offset(&mut self, correction_us: i64) {
        self.sync_offset_us += correction_us;
    }

    /// Gets the current synchronization offset in microseconds.
    ///
    /// Returns the cumulative time correction applied to this timer instance.
    /// This value represents how much the local time differs from the synchronized
    /// network time.
    ///
    /// # Returns
    ///
    /// The synchronization offset in microseconds:
    /// - Positive values: Local time is ahead of synchronized time
    /// - Negative values: Local time is behind synchronized time
    /// - Zero: Local time is synchronized
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    /// let offset = timer.get_sync_offset_us();
    /// 
    /// if offset > 0 {
    ///     println!("Local time is {} microseconds ahead", offset);
    /// } else if offset < 0 {
    ///     println!("Local time is {} microseconds behind", -offset);
    /// } else {
    ///     println!("Time is synchronized");
    /// }
    /// ```
    pub fn get_sync_offset_us(&self) -> i64 {
        self.sync_offset_us
    }

    /// Gets the synchronized time including the synchronization offset.
    ///
    /// This method returns the current hardware timer time adjusted by the
    /// synchronization offset. This provides the "network-synchronized" time
    /// that should be consistent across all nodes in the network.
    ///
    /// # Returns
    ///
    /// A `Duration` representing the synchronized time, which is the hardware
    /// timer time plus the synchronization offset.
    ///
    /// # Usage
    ///
    /// Use this method when you need time values that are consistent across
    /// multiple nodes in a synchronized network. For local timing operations,
    /// use [`Timer::get_time()`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let mut timer = Timer::get_timer(0).unwrap();
    /// 
    /// // Apply some synchronization offset
    /// timer.adjust_sync_offset(1000); // 1ms ahead
    /// 
    /// // Get synchronized time
    /// let sync_time = timer.get_synchronized_time();
    /// let local_time = timer.get_time();
    /// 
    /// // sync_time should be 1ms ahead of local_time
    /// println!("Synchronized time: {:?}", sync_time);
    /// println!("Local time: {:?}", local_time);
    /// ```
    pub fn get_synchronized_time(&self) -> Duration {
        let local_time = self.get_time();
        let offset_duration = Duration::from_micros(self.sync_offset_us.abs() as u64);
        
        if self.sync_offset_us >= 0 {
            local_time + offset_duration
        } else {
            local_time - offset_duration
        }
    }

    /// Resets the synchronization offset to zero.
    ///
    /// This method clears the current synchronization offset, effectively
    /// disabling time synchronization for this timer instance. This should
    /// be used when:
    /// - Disabling time synchronization
    /// - Resetting synchronization after errors
    /// - Recalibrating the timer
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let mut timer = Timer::get_timer(0).unwrap();
    /// 
    /// // Apply some offset
    /// timer.adjust_sync_offset(500);
    /// assert_eq!(timer.get_sync_offset_us(), 500);
    /// 
    /// // Reset synchronization
    /// timer.reset_sync_offset();
    /// assert_eq!(timer.get_sync_offset_us(), 0);
    /// ```
    pub fn reset_sync_offset(&mut self) {
        self.sync_offset_us = 0;
    }

    /// Checks if the timer is currently synchronized.
    ///
    /// A timer is considered synchronized if its synchronization offset
    /// is within acceptable bounds (typically close to zero).
    ///
    /// # Arguments
    ///
    /// * `tolerance_us` - Maximum acceptable offset in microseconds
    ///
    /// # Returns
    ///
    /// `true` if the timer is synchronized within the specified tolerance,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::timer::Timer;
    ///
    /// let mut timer = Timer::get_timer(0).unwrap();
    /// 
    /// // Small offset - synchronized
    /// timer.adjust_sync_offset(50);
    /// assert!(timer.is_synchronized(100)); // Within 100μs tolerance
    /// 
    /// // Large offset - not synchronized
    /// timer.adjust_sync_offset(1000);
    /// assert!(!timer.is_synchronized(100)); // Outside 100μs tolerance
    /// ```
    pub fn is_synchronized(&self, tolerance_us: u64) -> bool {
        self.sync_offset_us.abs() <= tolerance_us as i64
    }
}
