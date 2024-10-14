use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

/// The number of tasks can fit into a type usize.
pub type TaskNumberType = usize;
// TODO: rewrite with cfg!
#[cfg(not(feature = "c-library"))]
/// Type of setup function, that is called once at the beginning of task.
pub type TaskSetupFunctionType = fn() -> ();
#[cfg(feature = "c-library")]
/// Type of setup function, that is called once at the beginning of task.
type TaskSetupFunctionType = extern "C" fn() -> ();
#[cfg(not(feature = "c-library"))]
/// Type of loop function, that is called in loop.
pub type TaskLoopFunctionType = fn() -> ();
#[cfg(feature = "c-library")]
/// Type of loop function, that is called in loop.
type TaskLoopFunctionType = extern "C" fn() -> ();
#[cfg(not(feature = "c-library"))]
/// Type of condition function for stopping loop function execution.
pub type TaskStopConditionFunctionType = fn() -> bool;
#[cfg(feature = "c-library")]
/// Type of condition function for stopping loop function execution.
type TaskStopConditionFunctionType = extern "C" fn() -> bool;

#[repr(C)]
/// Task representation for task manager.
pub struct Task {
    /// Setup function, that is called once at the beginning of task.
    pub(crate) setup_fn: TaskSetupFunctionType,
    /// Loop function, that is called in loop.
    pub(crate) loop_fn: TaskLoopFunctionType,
    /// Condition function for stopping loop function execution.
    pub(crate) stop_condition_fn: TaskStopConditionFunctionType,
}

#[repr(C)]
/// Future shell for task for execution.
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
