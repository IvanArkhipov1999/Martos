//! # Cooperative Task Scheduler
//!
//! This module implements cooperative multitasking for Martos RTOS, providing priority-based
//! task scheduling where tasks voluntarily yield control to each other. The scheduler uses
//! a round-robin approach within priority levels and relies on tasks to cooperate rather
//! than being preemptively interrupted.
//!
//! # Architecture Overview
//!
//! The cooperative scheduler organizes tasks into 11 priority levels (0-10), where higher
//! numbers indicate higher priority. Within each priority level, tasks are scheduled using
//! round-robin to ensure fairness. Tasks must explicitly yield control by returning from
//! their loop function, making the system deterministic and suitable for real-time applications.
//!
//! ## Key Components
//!
//! - [`TaskStatusType`] - Enumeration of possible task states
//! - [`CooperativeTask`] - Individual task with state, priority, and ID
//! - [`CooperativeTaskManager`] - Main scheduler managing all tasks
//! - Priority queues organized as `[Option<Vec<CooperativeTask>>; 11]`
//!
//! ## Task State Machine
//!
//! ```text
//! [Ready] --schedule()--> [Running] --loop_fn_returns--> [Ready]
//!    |                       |                             ^
//!    |                  stop_condition()                   |
//!    |                       |                       wake_up()
//!    v                       v                             |
//! [Sleeping] <--put_to_sleep()   [Terminated] --deleted----+
//! ```
//!
//! ## Priority Scheduling
//!
//! Tasks are selected for execution in strict priority order. The scheduler always
//! chooses the highest priority ready task. Within the same priority level, tasks
//! are executed in round-robin fashion to ensure fair resource allocation.
//!
//! # Usage Examples
//!
//! ## Basic Task Management
//!
//! ```rust,no_run
//! use martos::task_manager::{TaskManager, TaskManagerTrait};
//! use core::sync::atomic::{AtomicU32, Ordering};
//!
//! static WORK_COUNTER: AtomicU32 = AtomicU32::new(0);
//!
//! fn sensor_setup() {
//!     println!("Initializing sensor task");
//! }
//!
//! fn sensor_loop() {
//!     let count = WORK_COUNTER.fetch_add(1, Ordering::Relaxed);
//!     println!("Reading sensor data... iteration {}", count);
//!     
//!     // Cooperative yield - function returns, allowing other tasks to run
//! }
//!
//! fn sensor_stop() -> bool {
//!     WORK_COUNTER.load(Ordering::Relaxed) >= 100
//! }
//!
//! // Add task with default priority (0)
//! TaskManager::add_task(sensor_setup, sensor_loop, sensor_stop);
//! TaskManager::start_task_manager();
//! ```
//!
//! ## Priority-Based Task Management
//!
//! ```
//! use martos::task_manager::{TaskManager, TaskManagerTrait};
//!
//! // Critical task with highest priority
//! TaskManager::add_priority_task(
//!     || println!("Critical task init"),
//!     || println!("Critical work"),
//!     || false,
//!     10  // Highest priority
//! );
//!
//! // Background task with lower priority
//! TaskManager::add_priority_task(
//!     || println!("Background task init"),
//!     || println!("Background processing"),
//!     || false,
//!     1   // Lower priority
//! );
//! ```
//!
//! # Performance Characteristics
//!
//! - **Scheduling complexity**: O(1) for task selection within priority level
//! - **Memory overhead**: O(n) where n is the number of tasks
//! - **Deterministic timing**: No unexpected interruptions or context switches
//! - **Priority inversion**: Possible if high-priority tasks wait on low-priority tasks
//!
//! # Safety Considerations
//!
//! ## Cooperative Nature
//! Tasks must be well-behaved and return control promptly. A task that never returns
//! from its loop function will prevent all other tasks from executing.
//!
//! ## Memory Safety
//! - Uses unsafe blocks to access global `TASK_MANAGER` state
//! - Task IDs may overflow (marked as TODO in implementation)
//! - No protection against use-after-free for task references
//!
//! ## Concurrency
//! - Designed for single-core systems only
//! - No built-in protection against race conditions between tasks
//!
//! # Limitations
//!
//! - **No preemption**: Misbehaving tasks can starve others
//! - **No automatic priority inheritance**: Priority inversion possible
//! - **Fixed priority levels**: Cannot add priorities beyond 0-10 range
//! - **ID overflow**: Task IDs will eventually wrap around
//!
//! # See Also
//!
//! - [`preemptive`] - Alternative preemptive scheduler
//! - [`TaskManagerTrait`] - Common interface for all schedulers
//! - [`Task`] - Basic task structure definition

extern crate alloc;

use crate::task_manager::{
    task::{Task, TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType},
    TaskManagerTrait, TASK_MANAGER,
};
use alloc::vec::Vec;

/// Type alias for task identifiers.
///
/// Each task receives a unique ID when created. IDs are assigned sequentially
/// starting from 1. The ID is used to reference tasks for operations like
/// sleeping, waking, and deletion.
///
/// # TODO
/// Handle ID overflow when `usize::MAX` tasks have been created.
type TaskIdType = usize;

/// Type alias for task priority values.
///
/// Priorities range from 0 to [`NUM_PRIORITIES`]-1, where higher values
/// indicate higher priority. Tasks with priority 10 will always be
/// scheduled before tasks with priority 9, and so on.
type TaskPriorityType = usize;

/// Number of priority levels supported by the scheduler.
///
/// The scheduler supports priorities from 0 (lowest) to 10 (highest).
/// This creates 11 distinct priority queues for task organization.
const NUM_PRIORITIES: usize = 11;

/// Represents the current execution state of a cooperative task.
///
/// Tasks transition between these states during their lifecycle as managed
/// by the scheduler and through explicit API calls.
///
/// # State Transitions
///
/// - **Ready → Running**: Task is selected by scheduler
/// - **Running → Ready**: Task loop function returns normally  
/// - **Running → Terminated**: Task stop condition becomes true
/// - **Ready → Sleeping**: Task is explicitly put to sleep
/// - **Sleeping → Ready**: Task is explicitly woken up
/// - **Any → Terminated**: Task is explicitly deleted
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TaskStatusType {
    /// Task is ready to be scheduled for execution.
    ///
    /// This is the initial state for newly created tasks and the state
    /// tasks return to after their loop function completes normally.
    Ready,

    /// Task is currently executing its loop function.
    ///
    /// Only one task can be in this state at any given time in the
    /// cooperative scheduler, as there is no preemption.
    Running,

    /// Task is sleeping and will not be scheduled.
    ///
    /// Tasks enter this state through explicit [`put_to_sleep`] calls
    /// and can only be scheduled again after [`wake_up_task`] is called.
    ///
    /// [`put_to_sleep`]: CooperativeTaskManager::put_to_sleep
    /// [`wake_up_task`]: CooperativeTaskManager::wake_up_task
    Sleeping,

    /// Task has finished execution and will be removed.
    ///
    /// Tasks enter this state when their stop condition returns `true`
    /// or when explicitly deleted. Terminated tasks are automatically
    /// removed from all scheduling queues.
    Terminated,
}

/// Represents a single cooperatively scheduled task.
///
/// A `CooperativeTask` wraps the user-provided callbacks stored in its `core`
/// and augments them with runtime metadata used by the cooperative scheduler:
/// a unique `id`, a lifecycle `status`, and a `priority` that determines when
/// the task is selected to run relative to others. The memory layout is
/// `repr(C)` to keep FFI options open and to ensure a stable layout across
/// compilers.
///
/// Lifecycle rules:
/// - `setup_fn` is executed exactly once when the task is created.
/// - `loop_fn` is called repeatedly by the scheduler while the task is `Running`.
///   Returning from `loop_fn` yields control cooperatively.
/// - After each `loop_fn` call, `stop_condition_fn` is evaluated; when it returns
///   `true`, the task transitions to `Terminated` and is removed from queues.
///
/// Scheduling rules:
/// - Higher `priority` values are scheduled before lower ones.
/// - Within the same `priority`, tasks are served in round-robin order.
#[repr(C)]
#[derive(Clone)]
pub struct CooperativeTask {
    /// Encapsulated task callbacks taken from the underlying `Task`:
    /// - `setup_fn`: runs once at creation for task-specific initialization
    /// - `loop_fn`: runs on each scheduling turn; returning yields cooperatively
    /// - `stop_condition_fn`: checked after `loop_fn`; `true` terminates the task
    pub(crate) core: Task,

    /// Monotonically increasing unique identifier assigned at creation.
    ///
    /// The first created task receives `id == 1`. The `id` is used by public
    /// APIs such as `put_to_sleep`, `wake_up_task`, `delete_task`, and
    /// `get_task_by_id` to reference a specific task.
    ///
    /// Note: `id` may eventually wrap if the system runs long enough; overflow
    /// handling is currently a known limitation.
    pub(crate) id: TaskIdType,

    /// Current lifecycle state of the task managed by the scheduler.
    ///
    /// Typical transitions: `Ready → Running → Ready` on each turn, `Running →
    /// Terminated` when `stop_condition_fn` returns `true`, and `Ready ↔ Sleeping`
    /// via explicit API calls. Externally mutating this field is intentionally
    /// restricted to the scheduler (`pub(crate)`).
    pub(crate) status: TaskStatusType,

    /// Priority used to order execution among tasks.
    ///
    /// Valid range is `0..NUM_PRIORITIES` (inclusive of 0, exclusive of the
    /// upper bound). Higher values indicate higher priority and are always
    /// scheduled before lower values. Within the same `priority`, the scheduler
    /// uses round-robin fairness. Changing priority after creation is not
    /// currently supported by this manager.
    pub(crate) priority: TaskPriorityType,
}

/// Cooperative task manager responsible for creating, organizing, and scheduling tasks.
///
/// The `CooperativeTaskManager` maintains per-priority ready queues and executes
/// tasks using strict priority selection with round-robin fairness within the
/// same priority. It does not preempt tasks; instead, tasks yield by returning
/// from their `loop_fn`. The manager also owns task identifiers and the notion of
/// the currently executing task.
///
/// Scheduling policy:
/// - Always pick the highest non-empty priority queue.
/// - Execute the task at the front of that queue.
/// - After a turn, move the task to the back of its queue unless it is still
///   `Running` or has been `Terminated`.
///
/// Invariants and notes:
/// - Each task has a unique, monotonically increasing `id` assigned at creation
///   time (starting from 1).
/// - `exec_task_id` is `Some(id)` only if a task with that `id` exists and is
///   either `Ready` or `Running`.
/// - Queues store only `Ready` or `Sleeping` tasks; `Terminated` tasks are
///   removed eagerly.
#[repr(C)]
pub struct CooperativeTaskManager {
    /// Per-priority ready queues containing `CooperativeTask` instances.
    ///
    /// The array length equals `NUM_PRIORITIES`. Each element is either `None`
    /// (no tasks exist at that priority) or `Some(Vec<...>)` storing tasks in
    /// scheduling order. Highest priority is the last index (`NUM_PRIORITIES-1`).
    pub(crate) tasks: [Option<Vec<CooperativeTask>>; NUM_PRIORITIES],

    /// Next unique identifier to assign on task creation.
    ///
    /// Starts from 0 internally and is incremented before assignment, so the
    /// first created task obtains `id == 1`. This value grows monotonically; ID
    /// wrap-around is a known limitation and not yet handled.
    pub(crate) next_task_id: TaskIdType,

    /// Identifier of the task selected for the current scheduling turn.
    ///
    /// If this field is `Some(id)`, the scheduler targets that task on the next call to
    /// `schedule()`. It may temporarily point to a `Sleeping` task; in that case
    /// the task is rotated to the end of its queue, and a new `exec_task_id` is
    /// selected from the highest non-empty priority.
    pub(crate) exec_task_id: Option<TaskIdType>,
}

/// Cooperative implementation of ```TaskManagerTrait```.
impl TaskManagerTrait for CooperativeTaskManager {
    /// Add a task to task manager. It should pass setup, loop, and condition functions.
    /// Task added with this function has ```priority``` 0.
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        CooperativeTaskManager::add_priority_task(setup_fn, loop_fn, stop_condition_fn, 0);
    }

    /// Start task manager work.
    fn start_task_manager() -> ! {
        loop {
            CooperativeTaskManager::schedule();
        }
    }
}

impl CooperativeTaskManager {
    /// Create new task manager.
    pub(crate) const fn new() -> CooperativeTaskManager {
        CooperativeTaskManager {
            tasks: [const { None }; NUM_PRIORITIES],
            next_task_id: 0,
            exec_task_id: None,
        }
    }

    /// Add a task to task manager. It should pass setup, loop, and condition functions.
    /// Task added with this function has given priority.
    pub fn add_priority_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
        priority: TaskPriorityType,
    ) {
        if priority >= NUM_PRIORITIES {
            panic!("Error: add_priority_task: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }

        let new_task =
            CooperativeTaskManager::create_task(setup_fn, loop_fn, stop_condition_fn, priority);
        (new_task.core.setup_fn)();
        CooperativeTaskManager::push_to_queue(new_task);

        unsafe {
            if TASK_MANAGER.exec_task_id.is_none() {
                TASK_MANAGER.exec_task_id = Some(TASK_MANAGER.next_task_id);
            }
        }
    }

    /// Helper function for ```add_priority_task```.
    fn create_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
        priority: TaskPriorityType,
    ) -> CooperativeTask {
        let task = Task {
            setup_fn,
            loop_fn,
            stop_condition_fn,
        };

        unsafe {
            // TODO: handling id overflow
            TASK_MANAGER.next_task_id += 1;
            let task_id = TASK_MANAGER.next_task_id;
            CooperativeTask {
                core: task,
                id: task_id,
                status: TaskStatusType::Ready,
                priority,
            }
        }
    }

    /// Task can put to sleep another task in ```Ready``` state by its ```id```.
    pub fn put_to_sleep(id: TaskIdType) {
        let Some(task) = CooperativeTaskManager::get_task_by_id(id) else {
            panic!("Error: put_to_sleep: Task with id {} not found.", id);
        };
        match task.status {
            TaskStatusType::Running => {
                panic!(
                    "Error: put_to_sleep: Task with id {} is currently running.",
                    id
                );
            }
            TaskStatusType::Sleeping => {
                panic!(
                    "Error: put_to_sleep: Task with id {} is currently sleeping.",
                    id
                );
            }
            TaskStatusType::Terminated => {
                panic!(
                    "Error: put_to_sleep: Task with id {} is terminated and will be removed soon.",
                    id
                );
            }
            TaskStatusType::Ready => {
                task.status = TaskStatusType::Sleeping;
            }
        }
    }

    /// Task can terminate and delete another task by ```id```.
    /// Remove task from ```tasks``` queue.
    pub fn delete_task(id: TaskIdType) {
        let Some(task) = CooperativeTaskManager::get_task_by_id(id) else {
            panic!("Error: delete_task: Task with id {} not found.", id);
        };
        let queue_opt = unsafe { TASK_MANAGER.tasks[task.priority].as_mut() };
        match queue_opt {
            Some(queue) => {
                if let Some(task_index) = queue.iter().position(|iter_task| task.id == iter_task.id)
                {
                    queue.remove(task_index);
                } else {
                    panic!(
                        "Error: delete_task: Task with id {} not found in the task list.",
                        task.id
                    );
                }
            }
            None => {
                panic!(
                    "Error:delete_task: Task with id {} does not exist in priority {}.",
                    task.id, task.priority
                );
            }
        }
    }

    /// Wake up task in ```Sleeping``` state. Otherwise, panic.
    pub fn wake_up_task(id: TaskIdType) {
        let Some(task) = CooperativeTaskManager::get_task_by_id(id) else {
            panic!("Error: wake_up_task: Task with id {} not found.", id);
        };
        if task.status != TaskStatusType::Sleeping {
            panic!(
                "Error: wake_up_task: Task with id {} is currently not sleeping.",
                id
            );
        }
        task.status = TaskStatusType::Ready;
    }

    /// Get a task by ```id``` and return it.
    pub fn get_task_by_id<'a>(id: TaskIdType) -> Option<&'a mut CooperativeTask> {
        unsafe {
            for queue in TASK_MANAGER.tasks.iter_mut().flatten() {
                if let Some(task) = queue.iter_mut().find(|task| task.id == id) {
                    return Some(task);
                }
            }
        }
        None
    }

    /// Get task ```id``` by its position in ```tasks``` vector.
    pub fn get_id_by_position(priority: TaskPriorityType, position: usize) -> TaskIdType {
        if priority >= NUM_PRIORITIES {
            panic!("Error: get_id_by_priorities: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }
        unsafe {
            if TASK_MANAGER.tasks[priority].is_none() {
                panic!(
                    "Error: get_id_by_position: No tasks found with priority {}.",
                    priority
                );
            }
            if TASK_MANAGER.tasks[priority].as_ref().unwrap().len() - 1 < position {
                panic!(
                    "Error: get_id_by_position: No tasks found for task on position {}.",
                    position
                );
            }
            TASK_MANAGER.tasks[priority]
                .as_ref()
                .unwrap()
                .get(position)
                .unwrap()
                .id
        }
    }

    /// Push task to the queue.
    fn push_to_queue(task: CooperativeTask) {
        let priority = task.priority;
        unsafe {
            if TASK_MANAGER.tasks[priority].is_none() {
                TASK_MANAGER.tasks[priority] = Some(Vec::new());
            }
            match TASK_MANAGER.tasks[priority].as_mut() {
                Some(queue) => {
                    queue.push(task);
                }
                None => {
                    panic!(
                        "Error: push_to_queue: Failed to push task to queue with priority {}.",
                        priority
                    );
                }
            }
        }
    }

    /// Get id of task to be executed next.
    fn get_next_task_id() -> Option<TaskIdType> {
        unsafe {
            for queue in TASK_MANAGER.tasks.iter_mut().rev().flatten() {
                if let Some(task) = queue.first() {
                    return Some(task.id);
                }
            }
        }
        None // In case when task manager has not tasks.
    }

    /// Push task to the other queue end.
    fn move_to_queue_end(task: &mut CooperativeTask) {
        unsafe {
            if let Some(queue) = TASK_MANAGER.tasks[task.priority].as_mut() {
                if let Some(task_index) = queue.iter().position(|iter_task| iter_task.id == task.id)
                {
                    let task = queue.remove(task_index);
                    queue.push(task);
                } else {
                    panic!(
                        "Error: move_to_queue_end: Can not find task with id {}.",
                        task.id
                    );
                }
            } else {
                panic!(
                    "Error: move_to_queue_end: Queue with priority {} is empty.",
                    task.priority
                );
            }
        }
    }

    /// One task manager iteration.
    pub fn schedule() {
        let exec_task_id_opt = unsafe { TASK_MANAGER.exec_task_id };
        if let Some(exec_task_id) = exec_task_id_opt {
            let Some(exec_task) = CooperativeTaskManager::get_task_by_id(exec_task_id) else {
                panic!("Error: schedule: Task with id {} not found.", exec_task_id);
            };
            match exec_task.status {
                TaskStatusType::Ready => {
                    exec_task.status = TaskStatusType::Running;
                }
                TaskStatusType::Running => {
                    (exec_task.core.loop_fn)();
                    let Some(exec_task) = CooperativeTaskManager::get_task_by_id(exec_task_id)
                    else {
                        panic!("Error: schedule: Task with id {} not found.", exec_task_id);
                    };
                    if (exec_task.core.stop_condition_fn)() {
                        exec_task.status = TaskStatusType::Terminated;
                        CooperativeTaskManager::delete_task(exec_task_id);

                        unsafe {
                            TASK_MANAGER.exec_task_id = CooperativeTaskManager::get_next_task_id();
                        }
                        return;
                    }
                }
                TaskStatusType::Sleeping => {
                    CooperativeTaskManager::move_to_queue_end(exec_task);
                }
                TaskStatusType::Terminated => {
                    CooperativeTaskManager::delete_task(exec_task_id);
                    return;
                }
            }
            if exec_task.status != TaskStatusType::Running {
                unsafe { TASK_MANAGER.exec_task_id = CooperativeTaskManager::get_next_task_id() }
            }
        }
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn test_start_task_manager() {
        for _n in 1..=1000 {
            CooperativeTaskManager::schedule();
        }
    }

    /// Reset task manager to default state.
    pub fn reset_task_manager() {
        unsafe {
            for priority in 0..NUM_PRIORITIES {
                if let Some(vec) = TASK_MANAGER.tasks[priority].as_mut() {
                    vec.clear();
                    TASK_MANAGER.tasks[priority] = None;
                }
            }
            TASK_MANAGER.next_task_id = 0;
            TASK_MANAGER.exec_task_id = None;
        }
    }

    /// Check if the task manager is empty.
    pub fn is_empty() -> bool {
        unsafe {
            for vec_opt in TASK_MANAGER.tasks.iter() {
                if vec_opt.is_some() {
                    return false;
                }
            }
            true
        }
    }

    /// Count tasks of the specified priority.
    #[cfg(feature = "cooperative_tests")]
    pub fn count_tasks_with_priority(priority: TaskPriorityType) -> usize {
        if priority >= NUM_PRIORITIES {
            panic!("Error: count_tasks_with_priority: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }
        unsafe {
            if let Some(vec) = TASK_MANAGER.tasks[priority].as_ref() {
                vec.len()
            } else {
                0
            }
        }
    }

    /// Count all tasks in task manager.
    #[cfg(feature = "cooperative_tests")]
    pub fn count_all_tasks() -> usize {
        unsafe {
            TASK_MANAGER
                .tasks
                .iter()
                .flatten() // Skip None
                .map(|vec| vec.len())
                .sum()
        }
    }

    /// Get task ```id```.
    #[cfg(feature = "cooperative_tests")]
    pub fn get_id_from_task(task: &mut CooperativeTask) -> TaskIdType {
        task.id
    }

    /// Get task's state.
    #[cfg(feature = "cooperative_tests")]
    pub fn get_status(task: &mut CooperativeTask) -> TaskStatusType {
        task.status
    }

    /// Get state ```Ready```.
    #[cfg(feature = "cooperative_tests")]
    pub fn ready_status() -> TaskStatusType {
        TaskStatusType::Ready
    }

    /// Get state ```Sleeping```.
    #[cfg(feature = "cooperative_tests")]
    pub fn sleeping_status() -> TaskStatusType {
        TaskStatusType::Sleeping
    }

    /// Get state ```Terminate```.
    #[cfg(feature = "cooperative_tests")]
    pub fn terminated_status() -> TaskStatusType {
        TaskStatusType::Terminated
    }

    /// Get state ```Running```.
    #[cfg(feature = "cooperative_tests")]
    pub fn running_status() -> TaskStatusType {
        TaskStatusType::Running
    }
}
