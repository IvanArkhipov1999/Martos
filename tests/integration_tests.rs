#[cfg(test)]
mod integration_tests {
    use std::thread;
    use std::time::Duration;
    use ma_rtos::timer::Timer;

    #[test]
    /// Tests work of several timers.
    fn test_several_timers() {
        let timer1 = Timer::new(0);
        let timer2 = Timer::new(10);

        timer1.start();
        thread::sleep(Duration::from_millis(2));
        let count1 = timer1.get_tick_counter();

        timer2.start();
        thread::sleep(Duration::from_millis(3));
        let count2 = timer2.get_tick_counter();

        timer2.stop();
        let count3 = timer1.get_tick_counter();
        timer1.stop();

        assert_eq!(count1, 1);
        assert_eq!(count2, 12);
        assert_eq!(count3, 4);
    }
}