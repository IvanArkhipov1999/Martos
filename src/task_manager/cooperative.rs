extern crate alloc;

use crate::task_manager::{
    task::{Task, TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType},
    TaskManagerTrait, TASK_MANAGER,
};
use alloc::vec::Vec;

/// The number of tasks id can fit into a type usize.
type TaskIdType = usize;
/// Type of priority number of a task.
type TaskPriorityType = usize;

/// Number of existing priorities.
const NUM_PRIORITIES: usize = 11;

/// The status of the task changes during execution. ```enum TaskStatusType``` contains possible states.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TaskStatusType {
    /// Task status after setup function. It is ready to be executed.
    Ready,
    /// Task status when loop function is running.
    Running,
    /// Task status when it is sleeping. After waking up, a task again starts loop_fn.
    Sleeping,
    /// Task status when it terminated.
    /// It can be in both cases when a task is finished and when the other task called
    /// ```terminate_task``` function with id of a task that will be terminated.
    Terminated,
}

/// The main structure for a cooperative task.
/// Shell for ```Task```, the same for both cooperative and preemptive task managers.
#[repr(C)]
pub struct CooperativeTask {
    ///  Contains 3 functions for task execution inherited from the ```Task```: ```setup_fn```,
    /// ```loop_fn``` and ```stop_condition_fn```.
    pub(crate) core: Task,
    /// Each task has a unique ```id```. The First ```id``` number is 0.
    pub(crate) id: TaskIdType,
    /// Status of existing ```CooperativeTask```. It may change during the task executing.
    pub(crate) status: TaskStatusType,
    /// Each ```CooperativeTask``` has a ```priority```.
    /// It is taken into account when selecting the next task to execute.
    pub(crate) priority: TaskPriorityType,
}

/// Cooperative task manager representation. Based on round-robin scheduling with priorities.
#[repr(C)]
pub struct CooperativeTaskManager {
    /// Array of vectors with ```CooperativeTask``` to execute.
    pub(crate) tasks: [Option<Vec<CooperativeTask>>; NUM_PRIORITIES],
    /// ```id``` of a task that will be created the next. The First task has id 1.
    pub(crate) next_task_id: TaskIdType,
    /// ```id``` of executing task.
    pub(crate) exec_task_id: TaskIdType,
}

/// Cooperative implementation of ```TaskManagerTrait```.
impl TaskManagerTrait for CooperativeTaskManager {
    /// Add a task to task manager. It should pass setup, loop, and condition functions.
    /// Task added with this function has ```priority``` 0.
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        CooperativeTaskManager::add_priority_task(setup_fn, loop_fn, stop_condition_fn, 0);
    }

    /// Start task manager work.
    fn start_task_manager() -> ! {
        loop {
            CooperativeTaskManager::schedule();
        }
    }
}

impl CooperativeTaskManager {
    /// Create new task manager.
    pub(crate) const fn new() -> CooperativeTaskManager {
        CooperativeTaskManager {
            tasks: [None; NUM_PRIORITIES],
            next_task_id: 0,
            exec_task_id: 0,
        }
    }

    /// Add a task to task manager. It should pass setup, loop, and condition functions.
    /// Task added with this function has given priority.
    pub fn add_priority_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
        priority: TaskPriorityType,
    ) {
        if priority >= NUM_PRIORITIES {
            panic!("Error: add_priority_task: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }

        let new_task =
            CooperativeTaskManager::create_task(setup_fn, loop_fn, stop_condition_fn, priority);
        (new_task.core.setup_fn)();
        CooperativeTaskManager::push_to_queue(new_task);

        unsafe {
            if TASK_MANAGER.exec_task_id == 0 {
                TASK_MANAGER.exec_task_id = TASK_MANAGER.next_task_id;
            }
        }
    }

    /// Helper function for ```add_priority_task```.
    fn create_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
        priority: TaskPriorityType,
    ) -> CooperativeTask {
        let task = Task {
            setup_fn,
            loop_fn,
            stop_condition_fn,
        };

        unsafe {
            TASK_MANAGER.next_task_id = TASK_MANAGER.next_task_id.wrapping_add(1);
            let task_id = TASK_MANAGER.next_task_id;
            CooperativeTask {
                core: task,
                id: task_id,
                status: TaskStatusType::Ready,
                priority,
            }
        }
    }

    /// Task can put to sleep another task in ```Ready``` state by its ```id```.
    pub fn put_to_sleep(id: TaskIdType) {
        let task = CooperativeTaskManager::get_task_by_id(id);
        match task.status {
            TaskStatusType::Running => {
                panic!("Error: put_to_sleep: Task with this id is currently running.");
            }
            TaskStatusType::Sleeping => {
                panic!("Error: put_to_sleep: Task with this id is currently sleeping.");
            }
            TaskStatusType::Terminated => {
                panic!(
                    "Error: put_to_sleep: Task with this id is terminated and soon will be removed."
                );
            }
            TaskStatusType::Ready => {
                task.status = TaskStatusType::Sleeping;
            }
        }
    }

    /// Task can terminate and delete another task by ```id```.ы
    pub fn terminate_task(id: TaskIdType) {
        let task = CooperativeTaskManager::get_task_by_id(id);
        CooperativeTaskManager::delete_task(task);
    }

    /// Wake up task in ```Sleeping``` state. Otherwise, panic.
    pub fn wake_up_task(id: TaskIdType) {
        let task = CooperativeTaskManager::get_task_by_id(id);
        if task.status != TaskStatusType::Sleeping {
            panic!(
                "Error: wake_up_task: Task with id {} is currently not sleeping.",
                id
            );
        }
        task.status = TaskStatusType::Ready;
    }

    /// Remove task from ```tasks``` queue.
    fn delete_task(task: &mut CooperativeTask) {
        unsafe {
            if let Some(vec) = TASK_MANAGER.tasks[task.priority].as_mut() {
                if let Some(pos) = vec.iter().position(|vec_task| vec_task.id == task.id) {
                    vec.remove(pos);
                } else {
                    panic!(
                        "Error: delete_task: Task with id {} not found in the task list.",
                        task.id
                    );
                }
            }
        }
    }

    /// Get a task by ```id``` and return it.
    pub fn get_task_by_id<'a>(id: TaskIdType) -> &'a mut CooperativeTask {
        unsafe {
            for vec_opt in TASK_MANAGER.tasks.iter_mut() {
                if let Some(vec) = vec_opt {
                    for task in vec.iter_mut() {
                        if task.id == id {
                            return task;
                        }
                    }
                }
            }
            panic!("Error: get_task_by_id: Task with id {} not found.", id);
        }
    }

    /// Get task ```id``` by its position in ```tasks``` vector.
    pub fn get_id_by_position(priority: TaskPriorityType, position: usize) -> TaskIdType {
        if priority >= NUM_PRIORITIES {
            panic!("Error: get_id_by_priorities: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }
        unsafe {
            if TASK_MANAGER.tasks[priority].is_none() {
                panic!(
                    "Error: get_id_by_position: No tasks found for priority {}.",
                    priority
                );
            }
            TASK_MANAGER.tasks[priority]
                .as_ref()
                .unwrap()
                .get(position)
                .unwrap()
                .id
        }
    }

    /// Push task to the queue.
    fn push_to_queue(task: CooperativeTask) {
        unsafe {
            let priority = task.priority;
            while TASK_MANAGER.tasks.len() <= priority {
                TASK_MANAGER.tasks.push(None);
            }

            if TASK_MANAGER.tasks[priority].is_none() {
                TASK_MANAGER.tasks[priority] = Some(Vec::new());
            }

            TASK_MANAGER.tasks[priority].as_mut().unwrap().push(task);
        }
    }

    /// Get id of task to be executed next.
    fn get_next_task_id() -> Option<TaskIdType> {
        unsafe {
            for opt_vec in TASK_MANAGER.tasks.iter_mut().rev() {
                if let Some(vec) = opt_vec {
                    if let Some(task) = vec.last_mut() {
                        return Some(task.id);
                    }
                }
            }
        }
        None
    }

    /// One task manager iteration.
    pub fn schedule() {
        if CooperativeTaskManager::is_empty() {
            let task_id = unsafe { TASK_MANAGER.exec_task_id };
            let task = CooperativeTaskManager::get_task_by_id(task_id);
            match task.status {
                TaskStatusType::Ready => {
                    task.status = TaskStatusType::Running;
                }
                TaskStatusType::Running => {
                    (task.core.loop_fn)();
                    if (task.core.stop_condition_fn)() {
                        task.status = TaskStatusType::Terminated;
                    }
                }
                TaskStatusType::Sleeping => {
                    // TODO: push_task_to_the_end_of_queue.
                }
                TaskStatusType::Terminated => {
                    CooperativeTaskManager::terminate_task(task_id);
                }
            }
            if task.status != TaskStatusType::Running {
                unsafe {
                    let Some(next_exec_id) = CooperativeTaskManager::get_next_task_id() else {
                        return;
                    };
                    unsafe { TASK_MANAGER.exec_task_id = next_exec_id }
                };
            }
        }
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn test_start_task_manager() {
        for _n in 1..=1000 {
            CooperativeTaskManager::schedule();
        }
    }

    /// Reset task manager to default state.
    pub fn reset_task_manager() {
        unsafe {
            for vec_opt in TASK_MANAGER.tasks.iter_mut() {
                let Some(vec) = vec_opt;
                vec.clear();
            }
            TASK_MANAGER.next_task_id = 0;
            TASK_MANAGER.exec_task_id = 0;
        }
    }

    /// Check if the task manager is empty.
    fn is_empty() -> bool {
        unsafe {
            TASK_MANAGER.tasks.iter().any(|opt_vec| {
                if let Some(vec) = opt_vec {
                    !vec.is_empty()
                } else {
                    false
                }
            })
        }
    }

    /// Count tasks of the specified priority.
    pub fn count_tasks_with_priority(priority: TaskPriorityType) -> usize {
        if priority >= NUM_PRIORITIES {
            panic!("Error: count_tasks_with_priority: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }
        unsafe {
            if let Some(vec) = TASK_MANAGER.tasks[priority].as_ref() {
                vec.len()
            } else {
                0
            }
        }
    }

    /// Count all tasks in task manager.
    pub fn count_all_tasks() -> usize {
        let mut sum: usize = 0;
        unsafe {
            for vec_opt in TASK_MANAGER.tasks.iter() {
                let Some(vec) = vec_opt;
                sum += vec.len();
            }
        }
        sum
    }

    /// Get task ```id```.
    pub fn get_id_from_task(task: &mut CooperativeTask) -> TaskIdType {
        task.id
    }

    /// Get task's state.
    pub fn get_status(task: &mut CooperativeTask) -> TaskStatusType {
        task.status
    }

    /// Get state ```Ready```.
    pub fn ready_status() -> TaskStatusType {
        TaskStatusType::Ready
    }

    /// Get state ```Sleeping```.
    pub fn sleeping_status() -> TaskStatusType {
        TaskStatusType::Sleeping
    }

    /// Get state ```Terminate```.
    pub fn terminated_status() -> TaskStatusType {
        TaskStatusType::Terminated
    }

    /// Get state ```Running```.
    pub fn running_status() -> TaskStatusType {
        TaskStatusType::Running
    }
}
