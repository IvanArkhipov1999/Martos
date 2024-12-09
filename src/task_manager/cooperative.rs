extern crate alloc;

use crate::task_manager::{
    task::{Task, TaskLoopFunctionType, TaskSetupFunctionType, TaskStopConditionFunctionType},
    TaskManager, TaskManagerTrait, TASK_MANAGER,
};
use alloc::vec::Vec;
use core::array;

type TaskIdType = usize;
type TaskPriorityType = usize;

const NUM_PRIORITIES: usize = 11;

#[derive(PartialEq)]
enum TaskStatusType {
    Created,
    Ready,
    Running,
    Sleep,
    Terminated,
}

#[repr(C)]
/// Future shell for a task for cooperative execution.
pub struct CooperativeTask {
    /// Task to execute in task manager.
    pub(crate) task_core: Task,
    id: TaskIdType,
    status: TaskStatusType,
    priority: TaskPriorityType,
}

#[repr(C)]
/// Task manager representation. Based on round-robin scheduling without priorities.
pub struct CooperativeTaskManager {
    /// Vector of tasks to execute.
    pub(crate) tasks: [Vec<CooperativeTask>; NUM_PRIORITIES],
    /// Index of a task, that should be executed.
    pub(crate) next_task_id: TaskIdType,
}

impl CooperativeTaskManager {
    /// Creates new task manager.
    pub(crate) const fn new() -> CooperativeTaskManager {
        CooperativeTaskManager {
            tasks: array::from_fn(|_| Vec::new()),
            next_task_id: 0,
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
                task_core: task,
                id: task_id,
                status: TaskStatusType::Created,
                priority,
            }
        }
    }

    fn push_to_queue(task: CooperativeTask) {
        unsafe {
            let mut task_vector = &TASK_MANAGER.tasks[task.priority];
            task_vector.push(task);
        }
    }

    fn setup_task(task: &mut CooperativeTask) {
        let res = (task.task_core.setup_fn)();
        if res == () {
            task.status = TaskStatusType::Ready
        }
        Err("Error: setup_task: setup_fn is invalid.");
    }

    pub fn add_priority_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
        priority: TaskPriorityType,
    ) {
        if priority <= 0 && priority >= NUM_PRIORITIES {
            Err("Error: add_task: Task's priority is invalid. It must be between 0 and 11.");
        }
        let mut new_task = TaskManager::create_task(setup_fn, loop_fn, stop_condition_fn, priority);
        TaskManager::setup_task(&mut new_task);
        TaskManager::push_to_queue(new_task);
    }

    pub unsafe fn find_task<'a>(id: TaskIdType) -> Result<&'a mut CooperativeTask, &'a str> {
        for vec in TASK_MANAGER.tasks.iter_mut() {
            for task in vec.iter_mut() {
                if task.id == id {
                    return Ok(task);
                }
            }
        }
        Err("Error: find_task: Task with this id not found.")
    }

    pub fn put_to_sleep(id: TaskIdType) {
        let res = unsafe { TaskManager::find_task(id) };
        if let Ok(mut task) = res {
            match task.status {
                TaskStatusType::Running => {
                    Err("Error: put_to_sleep: Task with this id is currently running.");
                }
                TaskStatusType::Sleep => {
                    Err("Error: put_to_sleep: Task with this id is currently sleeping.");
                }
                TaskStatusType::Terminated => {
                    Err("Error: put_to_sleep: Task with this id is terminated and recently will be removed.");
                }
                _ => {
                    task.status = TaskStatusType::Sleep;
                }
            }
        } else {
            Err(res.unwrap());
        }
    }

    pub fn terminate_task(id: TaskIdType) {
        let res = unsafe { TaskManager::find_task(id) };
        if let Ok(mut task) = res {
            task.status = TaskStatusType::Terminated;
            TaskManager::delete_task(task);
        } else {
            Err(res.unwrap());
        }
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

    fn get_next_task<'a>() -> Result<&'a mut CooperativeTask, &'a str> {
        unsafe {
            for vec in TASK_MANAGER.tasks.iter_mut() {
                if let Some(task) = vec.last_mut() {
                    return Ok(task);
                }
            }
        }
        Err("Error: get_next_task: No tasks currently, waiting for new tasks.")
    }

    /// Starts task manager work. Returns after 1000 steps only for testing task_manager_step.
    pub fn schedule() {
        if TaskManager::has_tasks() {
            let Ok(mut task) = TaskManager::get_next_task();
            match task.status {
                TaskStatusType::Created => {
                    TaskManager::setup_task(task);
                }
                TaskStatusType::Ready => {
                    task.status = TaskStatusType::Running;
                    (task.task_core.loop_fn)();
                }
                TaskStatusType::Running => {}
                TaskStatusType::Sleep => {}
                TaskStatusType::Terminated => {
                    TaskManager::delete_task(task);
                }
            }
        }
    }
}

impl TaskManagerTrait for CooperativeTaskManager {
    fn add_task(
        setup_fn: TaskSetupFunctionType,
        loop_fn: TaskLoopFunctionType,
        stop_condition_fn: TaskStopConditionFunctionType,
    ) {
        TaskManager::add_priority_task(setup_fn, loop_fn, stop_condition_fn, 0);
    }

    fn start_task_manager() -> ! {
        loop {
            TaskManager::schedule();
        }
    }
}
