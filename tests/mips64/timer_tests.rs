#[cfg(all(test, feature = "mips64_timer_tests"))]
mod timer_tests {
    use super::super::*;
    use crate::timer::TickType;
    use core::time::Duration;

    #[test]
    /// Tests the conversion from Duration to timer ticks.
    fn test_duration_to_ticks() {
        // Duration of 1 second and 1 microsecond into the expected number of ticks
        assert_eq!(
            duration_to_ticks(Duration::new(1, 1_000)),
            (TIMER_FREQUENCY * 1_000_001) as TickType
        );

        // Testing that the return value will saturate if it exceeds 64 bits
        assert_eq!(
            duration_to_ticks(Duration::from_micros(u64::MAX / TIMER_FREQUENCY + 1)),
            u64::MAX as TickType
        );
    }

    #[test]
    /// Tests the conversion from timer ticks to Duration.
    fn test_ticks_to_duration() {
        // Verifies that (1_000_001 * TIMER_FREQUENCY) ticks correspond to 1 second and 1 millisecond
        assert_eq!(
            ticks_to_duration((1_000_001 * TIMER_FREQUENCY) as TickType),
            Duration::new(1, 1_000)
        );

        // Tests that the maximum tick value is handled correctly when converted back to duration
        assert_eq!(
            ticks_to_duration(u64::MAX as TickType),
            Duration::from_micros(u64::MAX / TIMER_FREQUENCY)
        );
    }

    /// Helper function to check that all timers in a TimerBlock are in their default state.
    fn checking_for_untouched_timers<M: ByteAccess>(timer_block: &TimerBlock<M>) {
        let timers = [
            &timer_block.timer1,
            &timer_block.timer2,
            &timer_block.timer3,
            &timer_block.timer4,
        ];

        for timer in timers {
            assert_eq!(timer.duration, 0);
            assert_eq!(timer.reload_mode, false);
        }
    }

    // The following tests use a mock memory access implementation to simulate reading and writing to the timer configuration register

    #[test]
    /// Tests changing the operating mode of a timer to "on".
    fn test_change_operating_mode_on() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, 0x01B400088);
                0x80 // 10000000 - not 0x0 to check that the operation does not affect other digits
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, 0x01B400088);
                assert_eq!(value, 0x84); // 10000100
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.change_operating_mode(true);
        assert_eq!(timer_block.timer0.reload_mode, true);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    /// Tests changing the operating mode of a timer to "off".
    fn test_change_operating_mode_off() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, 0x01B400088);
                0x84 // 10000100 - not 0x4 (00000100) to check that the operation does not affect other digits
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, 0x01B400088);
                assert_eq!(value, 0x80); // 10000000
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.reload_mode = true;
        timer_block.timer0.change_operating_mode(false);
        assert_eq!(timer_block.timer0.reload_mode, false);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    /// Tests starting a timer.
    fn test_start_timer() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                0xC // 00001100 - not 0x0 to check that the operation does not affect other digits
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                assert_eq!(value, 0xD); // 00001101
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        // Sets the timer duration so that it starts
        timer_block.timer0.duration = 1;

        timer_block.timer0.start();

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    /// Tests loading a value into a timer.
    fn test_load_value() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, 0x01B400088);
                0xA0 // 10100000 - it is necessary that the 6th digit (01000000) is equal to zero
            }

            fn write_byte(&self, address: u64, value: u8) {
                match address {
                    0x01B400080 => assert_eq!(value, 0xAE),
                    0x01B400081 => assert_eq!(value, 0xD9),
                    0x01B400082 => assert_eq!(value, 0x78),
                    0x01B400083 => assert_eq!(value, 0x0),
                    0x01B400084 => assert_eq!(value, 0x0),
                    0x01B400085 => assert_eq!(value, 0x0),
                    0x01B400086 => assert_eq!(value, 0x0),
                    0x01B400087 => assert_eq!(value, 0x0),
                    _ => assert!(
                        false,
                        "Expected record at address {:X} with value {:X} is invalid.",
                        address, value
                    ),
                };
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        // 7920046:
        // - 01111000_11011001_10101110 in the binary number system
        // -     0x78     0xD9     0xAE - each byte is in hexadecimal notation
        timer_block.timer0.load_value(7920046 as TickType);
        assert_eq!(timer_block.timer0.duration, 7920046 as TickType);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    /// Tests the 'now' method, which calculates the current value of the timer.
    fn test_now() {
        #[derive(Clone)]
        struct MockMemoryAccess {
            counter: [u8; 8],
        }
        impl MockMemoryAccess {
            fn new() -> Self {
                Self {
                    counter: [0xAE, 0xD9, 0x78, 0x0, 0x0, 0x0, 0x0, 0x0], // 7920046
                }
            }
        }
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                match address {
                    0x01B400088 => 0x4, // 'Status and control register' of the timer - 00000100
                    0x01B400080..=0x01B400087 => {
                        let index = (address - 0x01B400080) as usize;
                        self.counter[index]
                    }
                    _ => {
                        assert!(false, "Unexpected address {:X}.", address);
                        0x0
                    }
                }
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, 0x01B400088);
                assert_eq!(value, 0x5) // 'Status and control register' of the timer after operation - 00000101
            }
        }

        let mock_memory_access = MockMemoryAccess::new();
        let mut timer_block = TimerBlock::new(mock_memory_access);

        // Sets the timer duration
        timer_block.timer0.duration = 7920052 as TickType;

        // When this function is called, the value on the counter is 7920046
        assert_eq!(timer_block.timer0.now(), 6 as TickType); // 7920052 - 7920046

        // Change the counter value to 7920035 and call the function with this value on the counter
        timer_block.timer0.accessibility.counter[0] = 0xA3;
        assert_eq!(timer_block.timer0.now(), 17 as TickType); // 7920052 - 7920035

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    /// Tests stopping a timer.
    fn test_stop_timer() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                0xD // 00001101 - not 0x1 to check that the operation does not affect other digits
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                assert_eq!(value, 0xC); // 00001100
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.stop();

        checking_for_untouched_timers(&timer_block);
    }
}
