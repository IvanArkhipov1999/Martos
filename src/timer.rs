use core::time::Duration;

use crate::ports::{Port, PortTrait};

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
/// // In your periodic interrupt or task:
/// timer.loop_timer(); // Update tick counter
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
    /// let mut timer = Timer::get_timer(0).unwrap();
    /// assert_eq!(timer.tick_counter, 0); // Starts at zero
    ///
    /// timer.loop_timer();
    /// assert_eq!(timer.tick_counter, 1); // Incremented by one
    /// ```
    pub tick_counter: TickType,
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
    /// use core::time::Duration;
    ///
    /// let timer = Timer::get_timer(0).unwrap();
    ///
    /// // Set various periods
    /// timer.change_period_timer(Duration::from_millis(100));  // 100ms
    /// timer.change_period_timer(Duration::from_micros(500));  // 500μs  
    /// timer.change_period_timer(Duration::from_secs(1));      // 1 second
    ///
    /// // Very precise timing (if hardware supports it)
    /// timer.change_period_timer(Duration::from_nanos(1250));  // 1.25μs
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
    /// let timer = Timer::get_timer(0).unwrap();
    /// timer.change_period_timer(Duration::from_millis(100));
    /// timer.start_timer();
    ///
    /// // Check current timer value
    /// let current_time = timer.get_time();
    /// println!("Timer value: {:?}", current_time);
    ///
    /// // Use for timing measurements
    /// let start_time = timer.get_time();
    /// // ... do some work ...
    /// let end_time = timer.get_time();
    /// let elapsed = end_time - start_time; // May need platform-specific calculation
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
    /// {
    ///     let timer = Timer::get_timer(0).unwrap();
    ///     
    ///     // Configure and use timer
    ///     timer.set_reload_mode(true);
    ///     timer.change_period_timer(Duration::from_millis(50));
    ///     timer.start_timer();
    ///     
    ///     // Do timing-related work...
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
}
