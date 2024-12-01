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

    fn checking_for_untouched_timers<M: ByteAccess>(timer_block: &TimerBlock<M>) {
        let timers = [
            &timer_block.timer1,
            &timer_block.timer2,
            &timer_block.timer3,
            &timer_block.timer4,
        ];

        for timer in timers {
            assert_eq!(timer.duration, 0);
            assert_eq!(timer.is_running, false);
            assert_eq!(timer.reload_mode, false);
        }
    }

    #[test]
    fn test_change_operating_mode_on() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, 0x01B400088);
                0x80 // Not 0x0 to check that the operation does not affect other bits
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, 0x01B400088);
                assert_eq!(value, 0x84);
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.change_operating_mode(true);
        assert_eq!(timer_block.timer0.reload_mode, true);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    fn test_change_operating_mode_off() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, 0x01B400088);
                0x84 // Not 0x4 to check that the operation does not affect other bits
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, 0x01B400088);
                assert_eq!(value, 0x80);
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.reload_mode = true;
        timer_block.timer0.change_operating_mode(false);
        assert_eq!(timer_block.timer0.reload_mode, false);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    fn test_start_timer() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                0xC // value - 00001100
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                assert_eq!(value, 0xD); // value - 00001101
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.duration = 1;
        timer_block.timer0.start();
        assert_eq!(timer_block.timer0.is_running, true);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    fn test_load_value() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, 0x01B400088);
                0xA0
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
                    0x01B400088 => 0x4, // STATUS_AND_CONTROL_REGISTER - 00000100
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
                assert_eq!(value, 0x5) // STATUS_AND_CONTROL_REGISTER after operation - 00000101
            }
        }

        let mock_memory_access = MockMemoryAccess::new();
        let mut timer_block = TimerBlock::new(mock_memory_access);

        timer_block.timer0.duration = 7920052 as TickType;
        assert_eq!(timer_block.timer0.now(), 6 as TickType); // 7920052 - 7920046

        timer_block.timer0.accessibility.counter[0] = 0xA3; // Change the counter value to 7920035
        assert_eq!(timer_block.timer0.now(), 17 as TickType);

        checking_for_untouched_timers(&timer_block);
    }

    #[test]
    fn test_stop_timer() {
        #[derive(Clone)]
        struct MockMemoryAccess;
        impl ByteAccess for MockMemoryAccess {
            fn read_byte(&self, address: u64) -> u8 {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                0xD // value - 00001101
            }

            fn write_byte(&self, address: u64, value: u8) {
                assert_eq!(address, CONFIGURATION_REGISTERS);
                assert_eq!(value, 0xC); // value - 00001100
            }
        }

        let mut timer_block = TimerBlock::new(MockMemoryAccess);

        timer_block.timer0.stop();
        assert_eq!(timer_block.timer0.is_running, false);

        checking_for_untouched_timers(&timer_block);
    }
}
