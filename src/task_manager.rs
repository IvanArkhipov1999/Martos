use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// The number of tasks can fit into a type usize.
pub type TaskNumberType = usize;

/// Task representation for task manager.
struct Task {
    /// Setup function, that is called once at the beginning of task.
    setup_fn: fn() -> (),
    /// Loop function, that is called in loop.
    loop_fn: fn() -> (),
    /// Condition for stopping loop function execution.
    stop_condition_fn: fn() -> bool,
}

/// Future shell for task for execution
struct FutureTask {
    /// Task to execute.
    task: Task,
    /// Marker for setup function completion.
    is_setup_completed: bool,
}

impl Future for FutureTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
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
fn task_waker() -> Waker {
    fn raw_clone(_: *const ()) -> RawWaker {
        RawWaker::new(0 as *const (), &NOOP_WAKER_VTABLE)
    }

    fn raw_wake(_: *const ()) {}

    fn raw_wake_by_ref(_: *const ()) {}

    fn raw_drop(_: *const ()) {}

    static NOOP_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

    let raw_waker = RawWaker::new(0 as *const (), &NOOP_WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

/// Task executor representation.
pub struct TaskExecutor {
    tasks: Vec<FutureTask>,
    task_to_execute_index: TaskNumberType,
}

impl TaskExecutor {
    /// Creates new task executor.
    pub fn new() -> TaskExecutor {
        TaskExecutor {
            tasks: vec![],
            task_to_execute_index: 0,
        }
    }

    /// Add task to task executor. You should pass setup and loop functions.
    pub fn add_task(
        &mut self,
        setup_fn: fn() -> (),
        loop_fn: fn() -> (),
        stop_condition_fn: fn() -> bool,
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
        self.tasks.push(future_task)
    }

    /// Starts task manager work.
    pub fn start_task_manager(&mut self) -> ! {
        loop {
            let waker = task_waker();
            let task = &mut self.tasks[self.task_to_execute_index];
            let mut task_future_pin = Pin::new(task);
            let _ = task_future_pin
                .as_mut()
                .poll(&mut Context::from_waker(&waker));

            if self.task_to_execute_index < self.tasks.len() - 1 {
                self.task_to_execute_index += 1;
            } else {
                self.task_to_execute_index = 0;
            }
        }
    }
}
