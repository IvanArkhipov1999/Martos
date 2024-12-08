extern crate alloc;

use crate::task_manager::{
    task::{Task, TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType},
    TaskManagerTrait, TASK_MANAGER,
};
use alloc::vec::Vec;
use core::task::{Poll, RawWaker, RawWakerVTable, Waker};
use core::{future::Future, pin::Pin, task::Context};

/// The number of tasks can fit into a type usize.
pub type TaskNumberType = usize;
#[repr(C)]
/// Future shell for task for cooperative execution.
pub struct FutureTask {
    /// Task to execute in task manager.
    pub(crate) task: Task,
    /// Marker for setup function completion.
    pub(crate) is_setup_completed: bool,
}

impl Future for FutureTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut array: [usize; 8] = core::array::from_fn(|i| i);
        array[0] = 5;
        if (self.task.stop_condition_fn)() {
            Poll::Ready(())
        } else {
            if !self.is_setup_completed {
                (self.task.setup_fn)();
                self.is_setup_completed = true;
            } else {
                (self.task.loop_fn)();
            }
            Poll::Pending
        }
    }
}

/// Creates simple task waker. May be more difficult in perspective.
pub fn task_waker() -> Waker {
    fn raw_clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null::<()>(), &NOOP_WAKER_VTABLE)
    }

    fn raw_wake(_: *const ()) {}

    fn raw_wake_by_ref(_: *const ()) {}

    fn raw_drop(_: *const ()) {}

    static NOOP_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

    let raw_waker = RawWaker::new(core::ptr::null::<()>(), &NOOP_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

#[repr(C)]
/// Task manager representation. Based on round-robin scheduling without priorities.
pub struct CooperativeTaskManager {
    /// Vector of tasks to execute.
    pub(crate) tasks: Vec<FutureTask>,
    /// Index of task, that should be executed.
    pub(crate) task_to_execute_index: TaskNumberType,
}

impl TaskManagerTrait for CooperativeTaskManager {
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        let task = Task {
            setup_fn,
            loop_fn,
            stop_condition_fn,
        };
        let future_task = FutureTask {
            task,
            is_setup_completed: false,
        };
        unsafe {
            TASK_MANAGER.tasks.push(future_task);
        }
    }

    fn start_task_manager() -> ! {
        loop {
            Self::task_manager_step();
        }
    }
}

impl CooperativeTaskManager {
    /// Creates new task manager.
    pub(crate) const fn new() -> CooperativeTaskManager {
        CooperativeTaskManager {
            tasks: Vec::new(),
            task_to_execute_index: 0,
        }
    }

    /// One step of task manager's work.
    // TODO: Support priorities.
    // TODO: Delete tasks from task vector if they are pending?
    fn task_manager_step() {
        if unsafe { !TASK_MANAGER.tasks.is_empty() } {
            let waker = task_waker();

            let task = unsafe { &mut TASK_MANAGER.tasks[TASK_MANAGER.task_to_execute_index] };
            let mut task_future_pin = Pin::new(task);
            let _ = task_future_pin
                .as_mut()
                .poll(&mut Context::from_waker(&waker));

            unsafe {
                if TASK_MANAGER.task_to_execute_index + 1 < TASK_MANAGER.tasks.len() {
                    TASK_MANAGER.task_to_execute_index += 1;
                } else {
                    TASK_MANAGER.task_to_execute_index = 0;
                }
            }
        }
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn test_start_task_manager() {
        for _n in 1..=1000 {
            Self::task_manager_step();
        }
    }
}
