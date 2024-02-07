#[cfg(test)]
mod integration_tests {
    use std::thread;
    use std::time::Duration;
    use ma_rtos::timer::Timer;

    #[test]
    /// Tests work of several timers.
    fn test_several_timers() {
        let timer1 = Timer::new(0, 10000, 0.1);
        let timer2 = Timer::new(10, 10000, 0.1);

        timer1.start();
        thread::sleep(Duration::from_millis(2));
        let count1 = timer1.get_tick_counter();

        timer2.start();
        thread::sleep(Duration::from_millis(3));
        let count2 = timer2.get_tick_counter();

        timer2.stop();
        let count3 = timer1.get_tick_counter();
        timer1.stop();

        // Exact values cannot be presented because of threads
        assert!(count1 <= 2);
        assert!(count2 <= 12);
        assert!(count3 <= 5);
    }
}