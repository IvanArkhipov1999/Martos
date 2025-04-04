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
///
#[repr(C)]
#[derive(Clone)]
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
    /// ```id``` of a task that will be created the next. The first created task has id 1.
    pub(crate) next_task_id: TaskIdType,
    /// ```id``` of executing task.
    pub(crate) exec_task_id: Option<TaskIdType>,
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
            tasks: [const { None }; NUM_PRIORITIES],
            next_task_id: 0,
            exec_task_id: None,
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
            if TASK_MANAGER.exec_task_id.is_none() {
                TASK_MANAGER.exec_task_id = Some(TASK_MANAGER.next_task_id);
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
            // TODO: handling id overflow
            TASK_MANAGER.next_task_id += 1;
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
        let Some(task) = CooperativeTaskManager::get_task_by_id(id) else {
            panic!("Error: put_to_sleep: Task with id {} not found.", id);
        };
        match task.status {
            TaskStatusType::Running => {
                panic!(
                    "Error: put_to_sleep: Task with id {} is currently running.",
                    id
                );
            }
            TaskStatusType::Sleeping => {
                panic!(
                    "Error: put_to_sleep: Task with id {} is currently sleeping.",
                    id
                );
            }
            TaskStatusType::Terminated => {
                panic!(
                    "Error: put_to_sleep: Task with id {} is terminated and will be removed soon.",
                    id
                );
            }
            TaskStatusType::Ready => {
                task.status = TaskStatusType::Sleeping;
            }
        }
    }

    /// Task can terminate and delete another task by ```id```.
    /// Remove task from ```tasks``` queue.
    pub fn delete_task(id: TaskIdType) {
        let Some(task) = CooperativeTaskManager::get_task_by_id(id) else {
            panic!("Error: delete_task: Task with id {} not found.", id);
        };
        let queue_opt = unsafe { TASK_MANAGER.tasks[task.priority].as_mut() };
        match queue_opt {
            Some(queue) => {
                if let Some(task_index) = queue.iter().position(|iter_task| task.id == iter_task.id)
                {
                    queue.remove(task_index);
                } else {
                    panic!(
                        "Error: delete_task: Task with id {} not found in the task list.",
                        task.id
                    );
                }
            }
            None => {
                panic!(
                    "Error:delete_task: Task with id {} does not exist in priority {}.",
                    task.id, task.priority
                );
            }
        }
    }

    /// Wake up task in ```Sleeping``` state. Otherwise, panic.
    pub fn wake_up_task(id: TaskIdType) {
        let Some(task) = CooperativeTaskManager::get_task_by_id(id) else {
            panic!("Error: wake_up_task: Task with id {} not found.", id);
        };
        if task.status != TaskStatusType::Sleeping {
            panic!(
                "Error: wake_up_task: Task with id {} is currently not sleeping.",
                id
            );
        }
        task.status = TaskStatusType::Ready;
    }

    /// Get a task by ```id``` and return it.
    pub fn get_task_by_id<'a>(id: TaskIdType) -> Option<&'a mut CooperativeTask> {
        unsafe {
            for queue in TASK_MANAGER.tasks.iter_mut().flatten() {
                if let Some(task) = queue.iter_mut().find(|task| task.id == id) {
                    return Some(task);
                }
            }
        }
        None
    }

    /// Get task ```id``` by its position in ```tasks``` vector.
    pub fn get_id_by_position(priority: TaskPriorityType, position: usize) -> TaskIdType {
        if priority >= NUM_PRIORITIES {
            panic!("Error: get_id_by_priorities: Task's priority {} is invalid. It must be between 0 and {}.", priority, NUM_PRIORITIES);
        }
        unsafe {
            if TASK_MANAGER.tasks[priority].is_none() {
                panic!(
                    "Error: get_id_by_position: No tasks found with priority {}.",
                    priority
                );
            }
            if TASK_MANAGER.tasks[priority].as_ref().unwrap().len() - 1 < position {
                panic!(
                    "Error: get_id_by_position: No tasks found for task on position {}.",
                    position
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
        let priority = task.priority;
        unsafe {
            if TASK_MANAGER.tasks[priority].is_none() {
                TASK_MANAGER.tasks[priority] = Some(Vec::new());
            }
            match TASK_MANAGER.tasks[priority].as_mut() {
                Some(queue) => {
                    queue.push(task);
                }
                None => {
                    panic!(
                        "Error: push_to_queue: Failed to push task to queue with priority {}.",
                        priority
                    );
                }
            }
        }
    }

    /// Get id of task to be executed next.
    fn get_next_task_id() -> Option<TaskIdType> {
        unsafe {
            for queue in TASK_MANAGER.tasks.iter_mut().rev().flatten() {
                if let Some(task) = queue.first() {
                    return Some(task.id);
                }
            }
        }
        None // In case when task manager has not tasks.
    }

    /// Push task to the other queue end.
    fn move_to_queue_end(task: &mut CooperativeTask) {
        unsafe {
            if let Some(queue) = TASK_MANAGER.tasks[task.priority].as_mut() {
                if let Some(task_index) = queue.iter().position(|iter_task| iter_task.id == task.id)
                {
                    let task = queue.remove(task_index);
                    queue.push(task);
                } else {
                    panic!(
                        "Error: move_to_queue_end: Can not find task with id {}.",
                        task.id
                    );
                }
            } else {
                panic!(
                    "Error: move_to_queue_end: Queue with priority {} is empty.",
                    task.priority
                );
            }
        }
    }

    /// One task manager iteration.
    pub fn schedule() {
        let exec_task_id_opt = unsafe { TASK_MANAGER.exec_task_id };
        if let Some(exec_task_id) = exec_task_id_opt {
            let Some(mut exec_task) = CooperativeTaskManager::get_task_by_id(exec_task_id) else {
                panic!("Error: schedule: Task with id {} not found.", exec_task_id);
            };
            match exec_task.status {
                TaskStatusType::Ready => {
                    exec_task.status = TaskStatusType::Running;
                }
                TaskStatusType::Running => {
                    (exec_task.core.loop_fn)();
                    let Some(exec_task) = CooperativeTaskManager::get_task_by_id(exec_task_id)
                    else {
                        panic!("Error: schedule: Task with id {} not found.", exec_task_id);
                    };
                    if (exec_task.core.stop_condition_fn)() {
                        exec_task.status = TaskStatusType::Terminated;
                        CooperativeTaskManager::delete_task(exec_task_id);

                        unsafe {
                            TASK_MANAGER.exec_task_id = CooperativeTaskManager::get_next_task_id();
                        }
                        return;
                    }
                }
                TaskStatusType::Sleeping => {
                    CooperativeTaskManager::move_to_queue_end(&mut exec_task);
                }
                TaskStatusType::Terminated => {
                    CooperativeTaskManager::delete_task(exec_task_id);
                    return;
                }
            }
            if exec_task.status != TaskStatusType::Running {
                unsafe { TASK_MANAGER.exec_task_id = CooperativeTaskManager::get_next_task_id() }
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
            for priority in 0..NUM_PRIORITIES {
                if let Some(vec) = TASK_MANAGER.tasks[priority].as_mut() {
                    vec.clear();
                    TASK_MANAGER.tasks[priority] = None;
                }
            }
            TASK_MANAGER.next_task_id = 0;
            TASK_MANAGER.exec_task_id = None;
        }
    }

    /// Check if the task manager is empty.
    pub fn is_empty() -> bool {
        unsafe {
            for vec_opt in TASK_MANAGER.tasks.iter() {
                if vec_opt.is_some() {
                    return false;
                }
            }
            true
        }
    }

    /// Count tasks of the specified priority.
    #[cfg(feature = "cooperative_tests")]
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
    #[cfg(feature = "cooperative_tests")]
    pub fn count_all_tasks() -> usize {
        unsafe {
            TASK_MANAGER
                .tasks
                .iter()
                .flatten() // Skip None
                .map(|vec| vec.len())
                .sum()
        }
    }

    /// Get task ```id```.
    #[cfg(feature = "cooperative_tests")]
    pub fn get_id_from_task(task: &mut CooperativeTask) -> TaskIdType {
        task.id
    }

    /// Get task's state.
    #[cfg(feature = "cooperative_tests")]
    pub fn get_status(task: &mut CooperativeTask) -> TaskStatusType {
        task.status
    }

    /// Get state ```Ready```.
    #[cfg(feature = "cooperative_tests")]
    pub fn ready_status() -> TaskStatusType {
        TaskStatusType::Ready
    }

    /// Get state ```Sleeping```.
    #[cfg(feature = "cooperative_tests")]
    pub fn sleeping_status() -> TaskStatusType {
        TaskStatusType::Sleeping
    }

    /// Get state ```Terminate```.
    #[cfg(feature = "cooperative_tests")]
    pub fn terminated_status() -> TaskStatusType {
        TaskStatusType::Terminated
    }

    /// Get state ```Running```.
    #[cfg(feature = "cooperative_tests")]
    pub fn running_status() -> TaskStatusType {
        TaskStatusType::Running
    }
}
