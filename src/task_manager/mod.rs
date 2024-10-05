extern crate alloc;

use core::future::Future;

pub mod task;

cfg_if::cfg_if! {
    if #[cfg(feature = "preemptive")] {
        mod preemptive;
        pub type TaskManager = preemptive::PreemptiveTaskManager;
    } else {
        mod cooperative;
        pub type TaskManager = cooperative::CooperativeTaskManager;
    }
}

/// Operating system task manager.
/// By default, [cooperative::CooperativeTaskManager] is used
static mut TASK_MANAGER: TaskManager = TaskManager::new();
