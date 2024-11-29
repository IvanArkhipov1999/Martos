extern crate alloc;

// A double-ended queue implemented with a growable ring buffer
use alloc::collections::VecDeque;
use core::array;

/// Type of loop function, that is called in loop.
#[cfg(not(feature = "c-library"))]
type TaskLoopFunctionType = fn();
#[cfg(feature = "c-library")]
type TaskLoopFunctionType = extern "C" fn();
type TaskIdType = usize;
type TaskPriorityType = usize;

const NUM_PRIORITIES: usize = 11;

enum TaskStatusType {
    Ready,
    Sleep,
    WokeUp,
    Terminated,
}

pub struct Task {
    id: TaskIdType,
    /// Loop function, that is called in loop.
    loop_fn: TaskLoopFunctionType,
    status: TaskStatusType,
    priority: TaskPriorityType,
}

struct TaskManager {
    priority_array: [VecDeque<Task>; NUM_PRIORITIES],
    next_task_id: TaskIdType,
}

impl TaskManager {
    fn new() -> Self {
        TaskManager {
            priority_array: array::from_fn(|_| VecDeque::new()),
            next_task_id: 0,
        }
    }

    fn create_task(&mut self, loop_fn: TaskLoopFunctionType, priority: TaskPriorityType) -> Task {
        self.next_task_id += 1;
        let id = self.next_task_id;
        Task {id, loop_fn, status: TaskStatusType::Ready, priority}
    }

    fn update_status(task: &mut Task, new_status: TaskStatusType) {
        task.status = new_status;
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
        let new_task = self.create_task(loop_fn, priority);
        self.push_to_queue(new_task);
        Ok(())
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
