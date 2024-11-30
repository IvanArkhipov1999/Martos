#[cfg(all(test, feature = "mips64_timer_tests"))]
mod timer_tests {
    use super::super::*;
    use crate::timer::TickType;
    use core::time::Duration;

    #[test]
    fn test_duration_to_ticks() {
        assert_eq!(
            duration_to_ticks(Duration::new(1, 1_000)),
            (TIMER_FREQUENCY * 1_000_001) as TickType
        );

        assert_eq!(
            duration_to_ticks(Duration::from_micros(u64::MAX / TIMER_FREQUENCY + 1)),
            u64::MAX as TickType
        );
    }

    #[test]
    fn test_ticks_to_duration() {
        assert_eq!(
            ticks_to_duration(4_000_004 as TickType),
            Duration::new(1, 1_000)
        );

        assert_eq!(
            ticks_to_duration(u64::MAX as TickType),
            Duration::from_micros(u64::MAX / 4)
        );
    }
}
