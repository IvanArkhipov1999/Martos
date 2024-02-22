#[cfg(test)]
mod unit_tests {
    use core::sync::atomic::{AtomicU32, Ordering};
    use ma_rtos::task_manager;

    #[test]
    /// Tests if task manager without tasks works during 1 second without panic.
    fn test_empty_task_manager() {
        let fun_thread = std::thread::spawn(|| {
            let mut task_executor = task_manager::TaskExecutor::new();
            task_executor.start_task_manager()
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(dbg!(fun_thread.is_finished()), false);
    }

    static COUNTER: AtomicU32 = AtomicU32::new(1);

    fn setup_fn() {}
    fn loop_fn() {
        COUNTER.fetch_add(1, Ordering::Relaxed);
    }

    fn stop_condition_fn() -> bool {
        let value = unsafe { COUNTER.as_ptr().read() };
        if value % 50 == 0 {
            return true;
        }
        return false;
    }

    #[test]
    /// Tests if task manager with task works during 1 second without panic.
    fn test_one_task_task_manager() {
        let fun_thread = std::thread::spawn(|| {
            let mut task_executor = task_manager::TaskExecutor::new();
            task_executor.add_task(setup_fn, loop_fn, stop_condition_fn);
            task_executor.start_task_manager()
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(dbg!(fun_thread.is_finished()), false);
        assert_eq!(unsafe { COUNTER.as_ptr().read() }, 50);
    }
}
