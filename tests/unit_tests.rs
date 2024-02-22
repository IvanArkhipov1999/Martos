#[cfg(test)]
mod unit_tests {
    use ma_rtos::task_manager;

    #[test]
    /// Tests if task manager without tasks works during 1 second without panic.
    fn test_task_manager() {
        let fun_thread = std::thread::spawn(|| {
            let mut task_executor = task_manager::TaskExecutor::new();
            task_executor.start_task_manager()
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(dbg!(fun_thread.is_finished()), false);
    }
}
