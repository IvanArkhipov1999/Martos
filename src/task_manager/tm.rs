use crate::ports::{Port, PortTrait, TrapFrame};
use crate::task_manager::task::{
    TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType,
};
use crate::task_manager::{TaskManagerTrait, TASK_MANAGER};
use alloc::vec::Vec;
use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};
use esp_println::println;

// const NUM_THREADS: usize = 8;
pub(crate) const THREAD_STACK_SIZE: usize = 1024; // TODO:

static mut NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Thread {
    /// Id of this thread
    id: usize,
    /// Pointer to the memory allocated for stack
    pub(crate) stack: *mut u8,
    /// **Arch specific** state of the registers at the moment of context switch
    pub(crate) context: TrapFrame,
    // Uninit threads to fill array in TM?
    // state: ThreadState
    pub(crate) func: fn() -> (),
}

impl Thread {
    fn new(stack: *mut u8, func: fn() -> ()) -> Self {
        let id = unsafe { NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed) };
        Thread {
            id,
            stack,
            context: TrapFrame::default(), // todo: default?
            func,
        }
    }
}

pub struct TM {
    // pub(crate) threads: [Thread; NUM_THREADS],
    pub(crate) tasks: Vec<Thread>,
    pub(crate) task_to_execute_index: usize,
}

impl TM {
    pub const fn new() -> Self {
        TM {
            // threads: [Thread::new(); NUM_THREADS],
            tasks: Vec::new(),
            task_to_execute_index: 0,
        }
    }

    /// Returns ref to task_to_execute_index thread (to save/load context)
    fn curr_thread() -> *const Thread {
        // todo: should work for array, but not for vec
        unsafe {
            TASK_MANAGER
                .tasks
                .as_ptr()
                .add(TASK_MANAGER.task_to_execute_index)
        }
    }

    fn next_thread() {
        // todo!("chooses the next thread and updates TM state")
        unsafe {
            TASK_MANAGER.task_to_execute_index =
                (TASK_MANAGER.task_to_execute_index + 1) % TASK_MANAGER.tasks.len()
        }
    }

    pub fn schedule(isr_ctx: &mut TrapFrame) {
        let ctx: &mut TrapFrame = &mut unsafe { (*Self::curr_thread()).context };
        Port::save_ctx(ctx, isr_ctx);

        Self::next_thread();

        let ctx: &TrapFrame = &unsafe { (*Self::curr_thread()).context };
        Port::load_ctx(ctx, isr_ctx);
    }
}

impl TaskManagerTrait for TM {
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        // todo!("takes task's closure, creates new thread, setups stack, adds to Queue");
        let align = 16; //todo: ?
        let layout = Layout::from_size_align(THREAD_STACK_SIZE, align).unwrap();
        let stack = unsafe { alloc::alloc::alloc(layout) };
        let func: fn() -> () = || loop {
            println!("loop")
        };
        // todo: ^^^ change task to single function?
        let mut thread = Thread::new(stack, func);
        Port::setup_stack(&mut thread);
        unsafe { TASK_MANAGER.tasks.push(thread) }
        // todo: dealloc
    }

    fn start_task_manager() -> ! {
        // todo!("call arch specific code to initialize periodic interrupt");
        // todo!("idle task?");
        Port::setup_interrupt();
        loop {}
    }
}
