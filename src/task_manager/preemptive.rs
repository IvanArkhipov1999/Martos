extern crate alloc;

use alloc::vec::Vec;
use core::{future::Future, pin::Pin, task::Context};
use crate::task_manager::{
    task,
    task::{
        FutureTask, Task, TaskLoopFunctionType, TaskNumberType, TaskSetupFunctionType, TaskStopConditionFunctionType,
    },
    TASK_MANAGER,
};

#[repr(C)]
/// Task manager representation. Based on round-robin scheduling without priorities.
pub struct PreemptiveTaskManager {
    /// Vector of tasks to execute.
    tasks: Vec<FutureTask>,
    /// Index of task, that should be executed.
    task_to_execute_index: TaskNumberType,
}

impl PreemptiveTaskManager {
    /// Creates new task manager.
    pub(crate) const fn new() -> PreemptiveTaskManager {
        todo!()
        // PreemptiveTaskManager {
        //     tasks: Vec::new(),
        //     task_to_execute_index: 0,
        // }
    }

    /// Add task to task manager. You should pass setup, loop and condition functions.
    pub fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        todo!()
        // let task = Task {
        //     setup_fn,
        //     loop_fn,
        //     stop_condition_fn,
        // };
        // let future_task = FutureTask {
        //     task,
        //     is_setup_completed: false,
        // };
        // unsafe {
        //     TASK_MANAGER.tasks.push(future_task);
        // }
    }

    /// One step of task manager's work.
    // TODO: Support priorities.
    // TODO: Delete tasks from task vector if they are pending?
    fn task_manager_step() {
        todo!()
        // if unsafe { !TASK_MANAGER.tasks.is_empty() } {
        //     let waker = task::task_waker();
        //
        //     let task = unsafe { &mut TASK_MANAGER.tasks[TASK_MANAGER.task_to_execute_index] };
        //     let mut task_future_pin = Pin::new(task);
        //     let _ = task_future_pin.as_mut().poll(&mut Context::from_waker(&waker));
        //
        //     unsafe {
        //         if TASK_MANAGER.task_to_execute_index + 1 < TASK_MANAGER.tasks.len() {
        //             TASK_MANAGER.task_to_execute_index += 1;
        //         } else {
        //             TASK_MANAGER.task_to_execute_index = 0;
        //         }
        //     }
        // }
    }

    /// Starts task manager work.
    pub fn start_task_manager() -> ! {
        todo!()
        // loop {
        //     PreemptiveTaskManager::task_manager_step();
        // }
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn test_start_task_manager() {
        todo!()
        // for _n in 1..=1000 {
        //     PreemptiveTaskManager::task_manager_step();
        // }
    }

}
