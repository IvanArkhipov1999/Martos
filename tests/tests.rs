#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use ma_rtos::timer::Timer;

    #[test]
    fn test_timer_start_stop() {
        let timer = Timer::new(0);
        timer.start();
        thread::sleep(Duration::from_millis(3));
        timer.stop();
        let count = timer.get_tick_counter();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_timer_running() {
        let timer = Timer::new(0);
        timer.start();
        thread::sleep(Duration::from_millis(2));
        let count = timer.get_tick_counter();

        assert!(count >= 1);
    }
}
