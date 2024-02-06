#[cfg(test)]
mod unit_tests {
    use std::thread;
    use std::time::Duration;
    use ma_rtos::timer::Timer;

    #[test]
    /// Tests new function of timer.
    fn test_timer_new() {
        let timer1 = Timer::new(0);
        let timer2 = Timer::new(42);
        let count1 = timer1.get_tick_counter();
        let count2 = timer2.get_tick_counter();

        assert_eq!(count1, 0);
        assert_eq!(count2, 42);
    }

    #[test]
    /// Tests start function of timer.
    fn test_timer_start() {
        let timer = Timer::new(0);
        timer.start();
        thread::sleep(Duration::from_millis(2));
        let count = timer.get_tick_counter();

        assert!(count <= 2);
    }

    #[test]
    /// Tests stop function of timer.
    fn test_timer_stop() {
        let timer = Timer::new(0);
        timer.start();
        thread::sleep(Duration::from_millis(3));
        timer.stop();
        let count = timer.get_tick_counter();

        assert!(count <= 3);
    }

    #[test]
    /// Tests get_tick_counter function of timer.
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
        assert!(count1 <= 3);
        assert!(count2 <= 7);
        assert!(count3 <= 7);
    }
}
