use crate::ports::{Port, PortTrait, TrapFrame, STACK_ALIGN};
use crate::task_manager::task::{
    Task, TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType,
};
use crate::task_manager::{TaskManagerTrait, TASK_MANAGER};
use alloc::vec::Vec;
use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};

pub(crate) const THREAD_STACK_SIZE: usize = 1024; // TODO:

static mut NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct Thread {
    /// id of this thread
    id: usize,
    /// Pointer to the memory allocated for stack
    pub(crate) stack: *mut u8,
    /// **Arch specific** state of the registers at the moment of context switch
    pub(crate) context: TrapFrame,
    pub(crate) func: fn(TaskSetupFunctionType, TaskLoopFunctionType, TaskStopConditionFunctionType),
    // TODO: use Task
    pub(crate) task: Task,
}

impl Thread {
    fn new(
        stack: *mut u8,
        func: fn(TaskSetupFunctionType, TaskLoopFunctionType, TaskStopConditionFunctionType),
        start: TaskSetupFunctionType,
        loop_: TaskLoopFunctionType,
        stop: TaskStopConditionFunctionType,
    ) -> Self {
        let id = unsafe { NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed) };
        Thread {
            id,
            stack,
            context: TrapFrame::default(),
            func,
            task: Task {
                setup_fn: start,
                loop_fn: loop_,
                stop_condition_fn: stop,
            },
        }
    }
}

pub struct PreemptiveTaskManager {
    pub(crate) tasks: Vec<Thread>,
    pub(crate) task_to_execute_index: usize,
}
static mut first: bool = true;

impl PreemptiveTaskManager {
    pub const fn new() -> Self {
        PreemptiveTaskManager {
            tasks: Vec::new(),
            task_to_execute_index: 0,
        }
    }

    fn next_thread() {
        unsafe {
            TASK_MANAGER.task_to_execute_index =
                (TASK_MANAGER.task_to_execute_index + 1) % TASK_MANAGER.tasks.len()
        }
    }

    pub fn schedule(isr_ctx: &mut TrapFrame) {
        if unsafe { !first } {
            let task = unsafe {
                TASK_MANAGER
                    .tasks
                    .get_mut(TASK_MANAGER.task_to_execute_index)
                    .unwrap()
            };
            let ctx = &mut task.context;
            Port::save_ctx(ctx, isr_ctx);

            Self::next_thread();
        }
        unsafe { first = false }

        let task = unsafe {
            TASK_MANAGER
                .tasks
                .get(TASK_MANAGER.task_to_execute_index)
                .unwrap()
        };
        let ctx = &task.context;
        Port::load_ctx(ctx, isr_ctx);
    }
}

fn thread_func(
    setup_fn: TaskSetupFunctionType,
    loop_fn: TaskLoopFunctionType,
    stop_fn: TaskStopConditionFunctionType,
) {
    setup_fn();
    loop {
        if stop_fn() {
            // TODO:
            loop {}
        } else {
            loop_fn();
        }
    }
}
impl TaskManagerTrait for PreemptiveTaskManager {
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        let layout = Layout::from_size_align(THREAD_STACK_SIZE, STACK_ALIGN).unwrap();
        let stack = unsafe { alloc::alloc::alloc(layout) };
        let mut thread = Thread::new(stack, thread_func, setup_fn, loop_fn, stop_condition_fn);
        Port::setup_stack(&mut thread);
        unsafe { TASK_MANAGER.tasks.push(thread) }
        // todo: dealloc
    }

    fn start_task_manager() -> ! {
        // todo!("idle task?");
        Port::setup_interrupt();
        loop {}
    }
}
