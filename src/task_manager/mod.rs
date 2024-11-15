extern crate alloc;

use core::future::Future;

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

/// Operating system task manager.
/// By default, [cooperative::CooperativeTaskManager] is used
static mut TASK_MANAGER: TaskManager = TaskManager::new();

pub trait TaskManagerTrait {
    /// Add task to task manager. You should pass setup, loop and condition functions.
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    );
    // fn add_task(
    //     setup_fn: TaskSetupFunctionType,
    //     loop_fn: TaskLoopFunctionType,
    //     stop_condition_fn: TaskStopConditionFunctionType,
    // ) {
    //     let task = Task {
    //         setup_fn,
    //         loop_fn,
    //         stop_condition_fn,
    //     };
    //     let future_task = FutureTask {
    //         task,
    //         is_setup_completed: false,
    //     };
    //     unsafe {
    //         TASK_MANAGER.tasks.push(future_task);
    //     }
    // }

    /// Starts task manager work.
    fn start_task_manager() -> !;
}
