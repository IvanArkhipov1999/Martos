extern crate alloc;

// A double-ended queue implemented with a growable ring buffer
use alloc::collections::VecDeque;
use core::array;
use crate::context_switcher::ContextSwitcher;

/// Type of loop function, that is called in loop.
#[cfg(not(feature = "c-library"))]
type TaskLoopFunctionType = fn();
#[cfg(feature = "c-library")]
type TaskLoopFunctionType = extern "C" fn();

type TaskPriorityType = usize;

const NUM_PRIORITIES: usize = 11;

enum TaskStatusType {
    Ready,
    Sleep,
    WokeUp,
    Terminated,
}

pub struct Task {
    /// Loop function, that is called in loop.
    loop_fn: TaskLoopFunctionType,
    status: TaskStatusType,
    priority: TaskPriorityType,
}

impl Task {
    pub fn new(loop_fn: TaskLoopFunctionType, priority: TaskPriorityType) -> Self {
        Task {loop_fn, status: TaskStatusType::Ready, priority}
    }

    fn update_status(&mut self, new_status: TaskStatusType) {
        self.status = new_status;
    }
}

struct TaskManager {
    priority_array: [VecDeque<Task>; NUM_PRIORITIES],
    switcher: ContextSwitcher,
}

impl TaskManager {
    fn new() -> TaskManager {
        TaskManager {
            priority_array: array::from_fn(|_| VecDeque::new()),
            switcher: ContextSwitcher::new(),
        }
    }

    fn pop_from_queue(&mut self, priority: TaskPriorityType) -> Option<Task> {
        self.priority_array[priority].pop_front()
    }

    fn push_to_queue(&mut self, task: Task) {
        self.priority_array[task.priority].push_back(task);
    }

    fn is_queue_empty(&self, priority: TaskPriorityType) -> bool {
        self.priority_array[priority].is_empty()
    }

    fn pop_next_task(&mut self) -> Option<Task> {
        for priority in (0 ..NUM_PRIORITIES).rev(){
            if self.is_queue_empty(priority) { continue; }
            let next_task = self.pop_from_queue(priority);
            return next_task;
        }
        None
    }

    pub fn add_task(
        &mut self,
        loop_fn: TaskLoopFunctionType,
        priority: TaskPriorityType,
    ) -> Result<(), &'static str> {
        if priority >= NUM_PRIORITIES {
            return Err("Invalid priority value");
        }
        let new_task = Task::new(loop_fn, priority);
        self.push_to_queue(new_task);
        Ok(())
    }

    pub fn yield_to_scheduler(&self, mut task: Task, new_status: TaskStatusType) {
        self.switcher.save_context(&mut task);
        task.update_status(new_status);
    }

    pub fn wake_up(&self, task: &mut Task) {
        self.switcher.load_context(task);
        task.update_status(TaskStatusType::Ready);
    }

    pub fn start_task_manager(&mut self) {
        loop {
            // if task is None, array is empty, waiting for new tasks in system
            let Some(mut task) = self.pop_next_task();
            match task.status {
                TaskStatusType::Ready => {
                    (task.loop_fn)();
                    if task.status = TaskStatusType::Sleep {
                        self.push_to_queue(task);
                    } // deleting task is not adding it back to the queue
                }
                TaskStatusType::Sleep => {
                    self.push_to_queue(task);
                }
                TaskStatusType::WokeUp => {
                    self.wake_up(&mut task);
                    task.status = TaskStatusType::Ready;
                    self.push_to_queue(task);
                }
                TaskStatusType::Terminated => {}
            }
        }
    }
}
