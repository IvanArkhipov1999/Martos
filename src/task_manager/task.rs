//! Task definitions and function types for Martos task management system.
//!
//! This module provides the core abstractions for defining tasks in Martos RTOS.
//! Tasks are composed of three main components: setup, loop, and stop condition functions.
//! The module supports both native Rust function types and C-compatible function types
//! for FFI integration.
//!
//! ## Task Lifecycle
//!
//! 1. **Setup Phase**: [`TaskSetupFunctionType`] is called once when the task starts
//! 2. **Execution Phase**: [`TaskLoopFunctionType`] is called repeatedly in a loop
//! 3. **Termination Check**: [`TaskStopConditionFunctionType`] is evaluated to determine when to stop
//!
//! ## C Compatibility
//!
//! When the `c-library` feature is enabled, all function types use `extern "C"` calling
//! convention for seamless integration with C code.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use martos::task_manager::{TaskManager, TaskManagerTrait};
//!
//! fn my_setup() {
//!     println!("Task starting...");
//! }
//!
//! fn my_loop() {
//!     println!("Task running...");
//! }
//!
//! fn my_stop_condition() -> bool {
//!     // Return true to stop the task
//!     false
//! }
//!
//! TaskManager::add_task(my_setup, my_loop, my_stop_condition);
//! TaskManager::start_task_manager();
//! ```

// TODO: rewrite with cfg! macro for cleaner conditional compilation
#[cfg(not(feature = "c-library"))]
/// Function type for task setup phase.
///
/// This function is called exactly once when a task begins execution, before
/// the main loop starts. Use this for initialization, resource allocation,
/// and any one-time setup operations required by the task.
///
/// # Calling Convention
///
/// Uses standard Rust calling convention when `c-library` feature is disabled.
///
/// # Examples
///
/// ```
/// use martos::task_manager::{TaskManager, TaskManagerTrait};
///
/// fn initialize_sensor() {
///     println!("Initializing sensor...");
///     // Hardware initialization code here
/// }
///
/// TaskManager::add_task(initialize_sensor, || {}, || false);
/// ```
///
/// # See Also
///
/// * [`TaskLoopFunctionType`] - The main execution function
/// * [`TaskStopConditionFunctionType`] - Termination condition
pub type TaskSetupFunctionType = fn() -> ();

#[cfg(feature = "c-library")]
/// Function type for task setup phase (C-compatible).
///
/// This function is called exactly once when a task begins execution, before
/// the main loop starts. Use this for initialization, resource allocation,
/// and any one-time setup operations required by the task.
///
/// # Calling Convention
///
/// Uses C calling convention (`extern "C"`) when `c-library` feature is enabled
/// for compatibility with C code and FFI.
///
/// # Examples
///
/// ```
/// // C code example
/// void my_task_setup(void) {
///     printf("Task setup from C\n");
///     // C initialization code
/// }
/// ```
///
/// # See Also
///
/// * [`TaskLoopFunctionType`] - The main execution function
/// * [`TaskStopConditionFunctionType`] - Termination condition
pub type TaskSetupFunctionType = extern "C" fn() -> ();

#[cfg(not(feature = "c-library"))]
/// Function type for task main execution loop.
///
/// This function is called repeatedly in a loop after the setup phase completes.
/// It should contain the main logic of the task. The function should execute
/// quickly and return control to the scheduler to maintain system responsiveness.
///
/// # Performance Considerations
///
/// - Keep execution time short to avoid blocking other tasks
/// - Avoid blocking operations that could freeze the scheduler
/// - Use cooperative multitasking principles
///
/// # Calling Convention
///
/// Uses standard Rust calling convention when `c-library` feature is disabled.
///
/// # Examples
///
/// ```
/// use martos::task_manager::{TaskManager, TaskManagerTrait};
/// use core::sync::atomic::{AtomicU32, Ordering};
///
/// static COUNTER: AtomicU32 = AtomicU32::new(0);
///
/// fn blink_led() {
///     let count = COUNTER.fetch_add(1, Ordering::Relaxed);
///     if count % 1000 == 0 {
///         println!("LED blink #{}", count);
///     }
/// }
///
/// TaskManager::add_task(|| {}, blink_led, || false);
/// ```
///
/// # See Also
///
/// * [`TaskSetupFunctionType`] - One-time initialization
/// * [`TaskStopConditionFunctionType`] - Loop termination condition
pub type TaskLoopFunctionType = fn() -> ();

#[cfg(feature = "c-library")]
/// Function type for task main execution loop (C-compatible).
///
/// This function is called repeatedly in a loop after the setup phase completes.
/// It should contain the main logic of the task. The function should execute
/// quickly and return control to the scheduler to maintain system responsiveness.
///
/// # Performance Considerations
///
/// - Keep execution time short to avoid blocking other tasks
/// - Avoid blocking operations that could freeze the scheduler
/// - Use cooperative multitasking principles
///
/// # Calling Convention
///
/// Uses C calling convention (`extern "C"`) when `c-library` feature is enabled
/// for compatibility with C code and FFI.
///
/// # Examples
///
/// ```
/// // C code example
/// void my_task_loop(void) {
///     // Main task logic in C
///     printf("Task loop iteration\n");
/// }
/// ```
///
/// # See Also
///
/// * [`TaskSetupFunctionType`] - One-time initialization
/// * [`TaskStopConditionFunctionType`] - Loop termination condition
pub type TaskLoopFunctionType = extern "C" fn() -> ();

#[cfg(not(feature = "c-library"))]
/// Function type for task termination condition.
///
/// This function is called by the task manager to determine whether the task
/// should continue running or terminate. Return `true` to stop the task,
/// `false` to continue execution.
///
/// # Return Value
///
/// * `true` - Task should terminate and be removed from the scheduler
/// * `false` - Task should continue running
///
/// # Calling Convention
///
/// Uses standard Rust calling convention when `c-library` feature is disabled.
///
/// # Examples
///
/// # Examples
///
/// ```
/// use martos::task_manager::{TaskManager, TaskManagerTrait};
/// use core::sync::atomic::{AtomicU32, Ordering};
///
/// static COUNTER: AtomicU32 = AtomicU32::new(0);
///
/// fn should_stop() -> bool {
///     let count = COUNTER.fetch_add(1, Ordering::Relaxed);
///     count >= 100  // Stop after 100 iterations
/// }
///
/// TaskManager::add_task(|| {}, || {}, should_stop);
/// ```
///
/// # See Also
///
/// * [`TaskSetupFunctionType`] - One-time initialization
/// * [`TaskLoopFunctionType`] - Main execution function
pub type TaskStopConditionFunctionType = fn() -> bool;

#[cfg(feature = "c-library")]
/// Function type for task termination condition (C-compatible).
///
/// This function is called by the task manager to determine whether the task
/// should continue running or terminate. Return `true` to stop the task,
/// `false` to continue execution.
///
/// # Return Value
///
/// * `true` - Task should terminate and be removed from the scheduler
/// * `false` - Task should continue running
///
/// # Calling Convention
///
/// Uses C calling convention (`extern "C"`) when `c-library` feature is enabled
/// for compatibility with C code and FFI.
///
/// # Examples
///
/// ```
/// // C code example
/// static int counter = 0;
///
/// bool my_stop_condition(void) {
///     counter++;
///     return counter >= 50;  // Stop after 50 iterations
/// }
/// ```
///
/// # See Also
///
/// * [`TaskSetupFunctionType`] - One-time initialization
/// * [`TaskLoopFunctionType`] - Main execution function
pub type TaskStopConditionFunctionType = extern "C" fn() -> bool;

/// Represents a task in the Martos task management system.
///
/// A task consists of three function pointers that define its behavior:
/// setup (initialization), loop (main execution), and stop condition (termination).
/// This structure enables cooperative multitasking where tasks voluntarily yield
/// control back to the scheduler.
///
/// # Memory Layout
///
/// The `#[repr(C)]` attribute ensures C-compatible memory layout for FFI integration.
///
/// # Cloning
///
/// Tasks can be cloned to create multiple instances with the same function pointers.
/// Note that this creates a shallow copy - the actual functions are not duplicated.
///
/// # Thread Safety
///
/// Tasks themselves are `Send` and `Sync` as they only contain function pointers.
/// However, the thread safety of task execution depends on the implementation
/// of the individual functions.
///
/// # Examples
///
/// ## Basic Task Creation
///
/// ```
/// use martos::task_manager::{TaskManager, TaskManagerTrait};
///
/// fn setup() {
///     println!("Task initializing...");
/// }
///
/// fn main_loop() {
///     println!("Task running...");
/// }
///
/// fn stop_condition() -> bool {
///     false // Run forever
/// }
///
/// TaskManager::add_task(setup, main_loop, stop_condition);
/// ```
///
/// ## Task with Termination Condition
///
/// ```
/// use martos::task_manager::{TaskManager, TaskManagerTrait};
/// use std::sync::atomic::{AtomicBool, Ordering};
///
/// static TASK_COMPLETE: AtomicBool = AtomicBool::new(false);
///
/// fn setup() {
///     println!("Starting timed task...");
/// }
///
/// fn work() {
///     // Do some work...
///     // Eventually set completion flag
///     TASK_COMPLETE.store(true, Ordering::Release);
/// }
///
/// fn is_complete() -> bool {
///     TASK_COMPLETE.load(Ordering::Acquire)
/// }
///
/// TaskManager::add_task(setup, work, is_complete);
/// ```
///
/// # Integration with TaskManager
///
/// Tasks are typically created and registered with the [`TaskManager`] which
/// handles their execution lifecycle:
///
/// ```rust,no_run
/// use martos::task_manager::{TaskManager, TaskManagerTrait};
///
/// // Create task functions
/// fn my_setup() { /* setup code */ }
/// fn my_loop() { /* main logic */ }
/// fn my_stop() -> bool { false }
///
/// // Register with task manager
/// TaskManager::add_task(my_setup, my_loop, my_stop);
/// TaskManager::start_task_manager();
/// ```
///
/// # TODO (needs update)
///
/// - Add support for task priorities and scheduling policies
/// - Implement task state tracking (running, suspended, terminated)
/// - Add task-specific data storage capabilities
/// - Support for task communication mechanisms (message passing, shared memory)
/// - Task dependency and synchronization primitives
#[repr(C)]
#[derive(Clone)]
pub struct Task {
    /// Setup function called once at task initialization.
    ///
    /// This function is invoked exactly once when the task is first started,
    /// before any loop iterations begin. Use this for:
    ///
    /// - Hardware initialization
    /// - Memory allocation
    /// - Resource acquisition
    /// - Initial state setup
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::task_manager::{TaskManager, TaskManagerTrait};
    ///
    /// fn my_setup() {
    ///     println!("Initializing task resources...");
    ///     // Initialize hardware, allocate memory, etc.
    /// }
    ///
    /// // Register task using public API
    /// TaskManager::add_task(my_setup, || {}, || false);
    /// ```
    /// ```
    pub(crate) setup_fn: TaskSetupFunctionType,

    /// Loop function called repeatedly during task execution.
    ///
    /// This is the main execution function of the task, called continuously
    /// until the stop condition returns `true`. Each invocation should:
    ///
    /// - Execute quickly to maintain system responsiveness
    /// - Perform a single unit of work
    /// - Avoid blocking operations
    /// - Yield control back to the scheduler promptly
    ///
    /// # Performance Guidelines
    ///
    /// - Keep execution time under 1ms for real-time systems
    /// - Use state machines for complex multi-step operations
    /// - Avoid infinite loops within the function
    /// - Consider using async/await patterns for I/O operations
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::task_manager::{TaskManager, TaskManagerTrait};
    /// use core::sync::atomic::{AtomicU32, Ordering};
    ///
    /// static COUNTER: AtomicU32 = AtomicU32::new(0);
    ///
    /// fn periodic_work() {
    ///     let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    ///     if count % 1000 == 0 {
    ///         println!("Processed {} items", count);
    ///     }
    /// }
    ///
    /// TaskManager::add_task(|| {}, periodic_work, || false);
    /// ```
    pub(crate) loop_fn: TaskLoopFunctionType,

    /// Stop condition function that determines task termination.
    ///
    /// This function is called by the task manager to check whether the task
    /// should continue running. It should execute quickly and return:
    ///
    /// - `true` to terminate the task
    /// - `false` to continue execution
    ///
    /// The task manager will remove terminated tasks from the execution queue.
    ///
    /// # Design Considerations
    ///
    /// - Keep the function lightweight and fast
    /// - Avoid side effects in the condition check
    /// - Use atomic operations for thread-safe state checking
    /// - Consider time-based, counter-based, or event-based termination
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::task_manager::{TaskManager, TaskManagerTrait};
    /// use core::sync::atomic::{AtomicBool, Ordering};
    ///
    /// static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
    ///
    /// fn should_terminate() -> bool {
    ///     SHUTDOWN_REQUESTED.load(Ordering::Acquire)
    /// }
    ///
    /// TaskManager::add_task(|| {}, || {}, should_terminate);
    /// ```
    pub(crate) stop_condition_fn: TaskStopConditionFunctionType,
}

// TODO: Implement additional task management features (needs update):
// - Task priorities and scheduling weights
// - Task state tracking (Ready, Running, Suspended, Terminated)
// - Task-local storage for per-task data
// - Task communication primitives (channels, shared memory)
// - Task dependency management
// - Resource usage monitoring and limits
// - Task profiling and performance metrics
