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

#[derive(PartialEq)]
enum TaskStatusType {
    Running,
    Ready,
    Sleep,
    Terminated,
}

pub struct Task {
    id: TaskIdType,
    /// Loop function, that is called in loop.
    loop_fn: TaskLoopFunctionType,
    status: TaskStatusType,
    priority: TaskPriorityType,
}

pub struct TaskManager {
    priority_array: [VecDeque<Task>; NUM_PRIORITIES],
    next_task_id: TaskIdType,
}

impl TaskManager {
    pub fn new() -> Self {
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

    fn find_task(&mut self, id: TaskIdType) -> Option<Task> {
        for queue in self.priority_array.iter_mut() {
            if let Some(pos) = queue.iter().position(|task| task.id == id) {
                return Some(queue.remove(pos).unwrap());
            }
        }
        None
    }

    pub fn put_to_sleep(&mut self, id: TaskIdType) -> Result<(), &'static str> {
        if let Some(mut task) = self.find_task(id) {
            if task.status == TaskStatusType::Running {
                return Err("Error: put_to_sleep: Task with this id is currently running")
            }
            if task.status != TaskStatusType::Ready {
                return Err("Error: put_to_sleep: Task with this id can not go to sleep");
            }
            task.status = TaskStatusType::Sleep;
            Ok(())
        } else {
            Err("Error: put_to_sleep: No task with that id")
        }
    }

    pub fn terminate_task(&mut self, id: TaskIdType) -> Result<(), &'static str> {
        match self.find_task(id) {
            None => {
                Err("Error: terminate_task: No task with that id")
            }
            Some(mut task) => {
                if task.status == TaskStatusType::Running {
                    return Err("Error: terminate_task: Task with this id is currently running")
                }
                if task.status != TaskStatusType::Ready {
                    return Err("Error: terminate_task: Task with this id can not go to sleep");
                }
                TaskManager::update_status(&mut task, TaskStatusType::Terminated);
                Ok(())
            }
        }
    }

    pub fn start_task_manager(&mut self) -> ! {
        loop {
            // if task is None, array is empty, waiting for new tasks in system
            let Some(mut task) = self.pop_next_task() else { continue };
            match task.status {
                TaskStatusType::Ready => {
                    task.status = TaskStatusType::Running;
                    (task.loop_fn)();
                    self.push_to_queue(task);
                }
                TaskStatusType::Sleep => {
                    self.push_to_queue(task);
                }
                TaskStatusType::Terminated => {}
                _ => {}
            }
        }
    }
}


// data strictures:
// - pub use binary_heap::BinaryHeap;
// - pub use btree_map::BTreeMap;
// - pub use btree_set::BTreeSet;
// - pub use linked_list::LinkedList;
// - pub use vec_deque::VecDeque;
// - Module vec
// - Module array


// если я верно понимаю, один объект был сделан, чтобы это был не совсем трейт, ибо в системе может
// быть только 1 экземпляр TaskManager, но всё равно мне кажется, что это не лучшее решение... уж
// лучше посмотреть, можно ли сделать синглтон, читаю эту штуку
// https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
// answers on question about unsafetyness of using 'static mut' in single-thread context https://users.rust-lang.org/t/is-static-mut-unsafe-in-a-single-threaded-context/94242/4
