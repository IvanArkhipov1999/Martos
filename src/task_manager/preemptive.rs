extern crate alloc;

use alloc::vec::Vec;

use crate::task_manager::{
    task::{FutureTask, TaskNumberType},
    TaskManagerTrait,
};

#[repr(C)]
/// Preemptive task manager representation. Based on round-robin scheduling without priorities.
pub struct PreemptiveTaskManager {
    /// Vector of tasks to execute.
    pub(crate) tasks: Vec<FutureTask>,
    /// Index of task, that should be executed.
    pub(crate) task_to_execute_index: TaskNumberType,
    // todo
    // time_quant: u32,
}

impl TaskManagerTrait for PreemptiveTaskManager {
    fn start_task_manager() -> ! {
        loop {
            Self::task_manager_step();
        }
    }
}
impl PreemptiveTaskManager {
    /// Creates new task manager.
    pub(crate) const fn new() -> PreemptiveTaskManager {
        PreemptiveTaskManager {
            tasks: Vec::new(),
            task_to_execute_index: 0,
        }
    }

    /// One step of task manager's work.
    // TODO: Support priorities.
    // TODO: Delete tasks from task vector if they are pending?
    fn task_manager_step() {
        todo!()
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    #[cfg(test)]
    pub fn test_start_task_manager() {
        for _n in 1..=1000 {
            Self::task_manager_step();
        }
    }
}
