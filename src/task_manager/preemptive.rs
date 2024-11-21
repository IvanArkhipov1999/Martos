use crate::ports::{Port, PortTrait, TrapFrame};
use crate::task_manager::task::{
    TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType,
};
use crate::task_manager::{TaskManagerTrait, TASK_MANAGER};
use alloc::vec::Vec;
use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};

pub(crate) const THREAD_STACK_SIZE: usize = 1024; // TODO:

static mut NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Thread {
    /// id of this thread
    id: usize,
    /// Pointer to the memory allocated for stack
    pub(crate) stack: *mut u8,
    /// **Arch specific** state of the registers at the moment of context switch
    pub(crate) context: TrapFrame,
    pub(crate) func: fn() -> (),
}

impl Thread {
    fn new(stack: *mut u8, func: fn() -> ()) -> Self {
        let id = unsafe { NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed) };
        Thread {
            id,
            stack,
            context: TrapFrame::default(),
            func,
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


impl TaskManagerTrait for PreemptiveTaskManager {
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        let align = 16; //todo: ?
        let layout = Layout::from_size_align(THREAD_STACK_SIZE, align).unwrap();
        let stack = unsafe { alloc::alloc::alloc(layout) };
        // todo: change task to single function?
        let mut thread = Thread::new(stack, loop_fn);
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
