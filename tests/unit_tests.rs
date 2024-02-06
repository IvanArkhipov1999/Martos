#[cfg(test)]
mod unit_tests {
    use std::thread;
    use std::time::Duration;
    use ma_rtos::timer::Timer;

    #[test]
    fn test_timer_start() {
        let timer = Timer::new(0);
        timer.start();
        thread::sleep(Duration::from_millis(2));
        let count = timer.get_tick_counter();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_timer_stop() {
        let timer = Timer::new(0);
        timer.start();
        thread::sleep(Duration::from_millis(3));
        timer.stop();
        let count = timer.get_tick_counter();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_timer_get_tick_counter() {
        let timer = Timer::new(0);
        timer.start();
        let count0 = timer.get_tick_counter();
        thread::sleep(Duration::from_millis(3));
        let count1 = timer.get_tick_counter();
        thread::sleep(Duration::from_millis(4));
        let count2 = timer.get_tick_counter();
        timer.stop();
        let count3 = timer.get_tick_counter();

        assert_eq!(count0, 0);
        assert_eq!(count1, 2);
        assert_eq!(count2, 6);
        assert_eq!(count3, 6);
    }
}
