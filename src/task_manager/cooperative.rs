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
#[derive(PartialEq)]
enum TaskStatusType {
    /// Task status after it is created.
    Created,
    /// Task status after setup function. It is ready to be executed.
    Ready,
    /// Task status when loop function is running.
    Running,
    /// Task status when it is sleeping.
    Sleeping,
    /// Task status when it terminated.
    /// It can be in both cases
    /// when a task is finished and when the other task called ```terminate_task``` function
    /// with id of a task that will be terminated.
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
    id: TaskIdType,
    /// Status of existing ```CooperativeTask```. It may change during the task executing.
    status: TaskStatusType,
    /// Each ```CooperativeTask``` has a ```priority```.
    /// It is taken into account when selecting the next task to execute.
    priority: TaskPriorityType,
}

/// Cooperative task manager representation. Based on round-robin scheduling with priorities.
#[repr(C)]
pub struct CooperativeTaskManager {
    /// Array of vectors with ```CooperativeTask``` to execute.
    pub(crate) tasks: [Vec<CooperativeTask>; NUM_PRIORITIES],
    /// ```id``` of a task that will be created the next.
    pub(crate) next_task_id: TaskIdType,
}

/// Cooperative implementation of ```TaskManagerTrait```.
impl TaskManagerTrait for CooperativeTaskManager {
    /// Add a task to task manager.
    /// It should pass setup, loop, and condition functions.
    /// Task added with this function has ```priority``` 0.
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        CooperativeTaskManager::add_priority_task(setup_fn, loop_fn, stop_condition_fn, 0);
    }

    /// Starts task manager work.
    fn start_task_manager() -> ! {
        loop {
            CooperativeTaskManager::schedule();
        }
    }
}

impl CooperativeTaskManager {
    /// Creates new task manager.
    pub(crate) const fn new() -> CooperativeTaskManager {
        let tasks = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];
        CooperativeTaskManager {
            tasks,
            next_task_id: 0,
        }
    }

    /// Add a task to task manager.
    /// It should pass setup, loop, and condition functions.
    /// Task added with this function has given priority.
    pub fn add_priority_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
        priority: TaskPriorityType,
    ) {
        if priority >= NUM_PRIORITIES {
            panic!("Error: add_task: Task's priority is invalid. It must be between 0 and 11.");
        }
        let mut new_task =
            CooperativeTaskManager::create_task(setup_fn, loop_fn, stop_condition_fn, priority);
        CooperativeTaskManager::setup_task(&mut new_task);
        CooperativeTaskManager::push_to_queue(new_task);
    }

    /// Find a task by ```id``` and return it.
    pub unsafe fn find_task<'a>(id: TaskIdType) -> &'a mut CooperativeTask {
        for vec in TASK_MANAGER.tasks.iter_mut() {
            for task in vec.iter_mut() {
                if task.id == id {
                    return task;
                }
            }
        }
        panic!("Error: find_task: Task with this id not found.");
    }

    /// Task can put to sleep another task by ```id```.
    pub fn put_to_sleep(id: TaskIdType) {
        let res = unsafe { CooperativeTaskManager::find_task(id) };
        let task = res;
        match task.status {
            TaskStatusType::Running => {
                panic!("Error: put_to_sleep: Task with this id is currently running.");
            }
            TaskStatusType::Sleeping => {
                panic!("Error: put_to_sleep: Task with this id is currently sleeping.");
            }
            TaskStatusType::Terminated => {
                panic!(
                    "Error: put_to_sleep: Task with this id is terminated and recently will be removed."
                );
            }
            _ => {
                task.status = TaskStatusType::Sleeping;
            }
        }
    }

    /// Task can terminate and delete another task by ```id``` even if it executes.
    pub fn terminate_task(id: TaskIdType) {
        let res = unsafe { CooperativeTaskManager::find_task(id) };
        let task = res;
        task.status = TaskStatusType::Terminated;
        CooperativeTaskManager::delete_task(task);
    }

    /// One task manager iteration.
    pub fn schedule() {
        if CooperativeTaskManager::has_tasks() {
            let task = CooperativeTaskManager::get_next_task();
            match task.status {
                TaskStatusType::Created => {
                    CooperativeTaskManager::setup_task(task);
                }
                TaskStatusType::Ready => {
                    task.status = TaskStatusType::Running;
                    (task.core.loop_fn)();
                }
                TaskStatusType::Running => {}
                TaskStatusType::Sleeping => {}
                TaskStatusType::Terminated => {
                    if (task.core.stop_condition_fn)() {
                        CooperativeTaskManager::delete_task(task);
                    } else {
                        task.status = TaskStatusType::Ready;
                    }
                }
            }
        }
    }

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
            TASK_MANAGER.next_task_id += 1;
            let task_id = TASK_MANAGER.next_task_id;
            CooperativeTask {
                core: task,
                id: task_id,
                status: TaskStatusType::Created,
                priority,
            }
        }
    }

    fn push_to_queue(task: CooperativeTask) {
        unsafe {
            let task_vector = &mut TASK_MANAGER.tasks[task.priority];
            task_vector.push(task);
        }
    }

    fn setup_task(task: &mut CooperativeTask) {
        (task.core.setup_fn)();
        task.status = TaskStatusType::Ready;
    }

    fn delete_task(task: &mut CooperativeTask) {
        unsafe {
            let vec = &mut TASK_MANAGER.tasks[task.priority];
            if let Some(pos) = vec.iter().position(|vec_task| vec_task.id == task.id) {
                vec.remove(pos);
            }
        }
    }

    fn has_tasks() -> bool {
        unsafe {
            for vec in TASK_MANAGER.tasks.iter() {
                if !vec.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    fn get_next_task<'a>() -> &'a mut CooperativeTask {
        unsafe {
            for vec in TASK_MANAGER.tasks.iter_mut() {
                if let Some(task) = vec.last_mut() {
                    return task;
                }
            }
        }
        panic!("Error: get_next_task: No tasks currently, waiting for new tasks.");
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn test_start_task_manager() {
        for _n in 1..=1000 {
            CooperativeTaskManager::schedule();
        }
    }
}
