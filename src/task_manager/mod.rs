//! Task Manager Module for Martos RTOS
//!
//! This module provides the core task management infrastructure for Martos, including
//! the main task manager abstraction and the common interface for task operations.
//! The actual task manager implementation is selected at compile time based on
//! feature flags.
//!
//! # Architecture
//!
//! The module uses a compile-time strategy pattern to select between different
//! task scheduling implementations:
//!
//! - **Cooperative Scheduler** (default): Round-robin scheduling with 11 priority levels
//! - **Preemptive Scheduler** (with `preemptive` feature): Hardware-assisted context switching
//!
//! # Core Components
//!
//! ## TaskManager
//! Type alias that resolves to the active task manager implementation based on feature flags.
//! This provides a unified interface regardless of the underlying scheduler.
//!
//! ## TaskManagerTrait
//! Common interface that both cooperative and preemptive schedulers implement,
//! ensuring consistent API across different scheduling strategies.
//!
//! ## TASK_MANAGER
//! Static singleton instance of the currently active task manager, used internally
//! by the Martos kernel for task scheduling operations.
//!
//! # Feature Selection
//!
//! ```toml
//! # Cargo.toml
//! [features]
//! preemptive = []  # Enable preemptive scheduling
//! ```
//!
//! - **Without `preemptive`**: Uses `CooperativeTaskManager` with priority-based round-robin
//! - **With `preemptive`**: Uses `PreemptiveTaskManager` with stack-based context switching
//!
//! # Usage Examples
//!
//! ## Basic Task Creation
//!
//! ```rust,no_run
//! use martos::task_manager::{TaskManager, TaskManagerTrait};
//!
//! fn setup_sensor() {
//!     println!("Initializing sensor...");
//! }
//!
//! fn read_sensor() {
//!     println!("Reading sensor data...");
//! }
//!
//! fn stop_condition() -> bool {
//!     false // Run forever
//! }
//!
//! // Add task to scheduler
//! TaskManager::add_task(setup_sensor, read_sensor, stop_condition);
//!
//! // Start the scheduler (never returns)
//! TaskManager::start_task_manager();
//! ```
//!
//! # Design Benefits
//!
//! - **Compile-time Selection**: Zero runtime overhead for scheduler choice
//! - **Unified API**: Same interface regardless of underlying implementation
//! - **Extensible**: Easy to add new scheduler types in the future
//! - **Type Safety**: Compile-time guarantees about scheduler capabilities
//!
//! # See Also
//!
//! - [`task`] - Basic task definitions and function types
//! - [`cooperative`] - Cooperative scheduler with priority support
//! - [`preemptive`] - Preemptive scheduler with context switching
//! - [`TaskManagerTrait`] - Common interface for all task managers

extern crate alloc;

use crate::task_manager::task::{
    TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType,
};

mod task;

cfg_if::cfg_if! {
    if #[cfg(feature = "preemptive")] {
        pub(crate) mod preemptive;
        pub type TaskManager = preemptive::PreemptiveTaskManager;
    } else {
        mod cooperative;
        pub type TaskManager = cooperative::CooperativeTaskManager;
    }
}

/// Global task manager instance used by the Martos kernel.
///
/// This static mutable variable provides the active task manager implementation.
/// All access to this variable must be wrapped in `unsafe` blocks, as required
/// by Rust for `static mut` variables.
///
/// # Safety
///
/// Callers must ensure that access to this variable is properly synchronized
/// to avoid data races. In Martos, this is achieved through the single-threaded
/// embedded environment and careful task scheduling logic.
static mut TASK_MANAGER: TaskManager = TaskManager::new();

/// Common interface for all task manager implementations.
///
/// This trait defines the core operations that every task manager must support,
/// ensuring a consistent API regardless of the underlying scheduling strategy.
/// Both cooperative and preemptive schedulers implement this trait.
///
/// # Required Methods
///
/// - [`add_task`](TaskManagerTrait::add_task) - Register a new task with the scheduler
/// - [`start_task_manager`](TaskManagerTrait::start_task_manager) - Begin task execution
///
/// # Implementation Notes
///
/// Implementations of this trait are expected to:
/// - Handle task lifecycle management (creation, execution, termination)
/// - Provide appropriate scheduling behavior for their strategy
/// - Ensure memory safety and proper resource cleanup
/// - Support the three-phase task model (setup, loop, stop_condition)
pub trait TaskManagerTrait {
    /// Register a new task with the scheduler.
    ///
    /// Takes three function pointers defining the task's behavior:
    /// - `setup_fn`: Called once when the task starts
    /// - `loop_fn`: Called repeatedly during task execution  
    /// - `stop_condition_fn`: Determines when the task should terminate
    ///
    /// # Arguments
    ///
    /// * `setup_fn` - Initialization function called once at task start
    /// * `loop_fn` - Main execution function called repeatedly
    /// * `stop_condition_fn` - Termination condition check function
    ///
    /// # Examples
    ///
    /// ```
    /// use martos::task_manager::{TaskManager, TaskManagerTrait};
    ///
    /// fn init() { println!("Task starting"); }
    /// fn work() { println!("Task working"); }  
    /// fn done() -> bool { false }
    ///
    /// TaskManager::add_task(init, work, done);
    /// ```
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    );

    /// Start the task scheduler and begin executing tasks.
    ///
    /// This function begins the main scheduling loop and never returns.
    /// All registered tasks will be executed according to the scheduler's
    /// strategy (cooperative round-robin or preemptive time-slicing).
    ///
    /// # Behavior
    ///
    /// - **Cooperative**: Tasks run until they voluntarily yield control
    /// - **Preemptive**: Tasks are interrupted and switched based on timer events
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use martos::task_manager::{TaskManager, TaskManagerTrait};
    ///
    /// // Register tasks first
    /// TaskManager::add_task(|| {}, || {}, || false);
    ///
    /// // Start scheduler - this never returns!
    /// TaskManager::start_task_manager();
    /// ```
    fn start_task_manager() -> !;
}
