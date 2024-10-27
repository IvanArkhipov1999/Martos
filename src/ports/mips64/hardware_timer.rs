use crate::timer::TickType;

// static mut TIMER0:
static mut TIMER0: Option<Timer> = None;

const TIMER_0: u64 = 0x01B400080;
const TIMER_1: u64 = 0x01B400090;
const TIMER_2: u64 = 0x01B4000A0;
const TIMER_3: u64 = 0x01B4000B0;
const TIMER_4: u64 = 0x01B4000C0;
const CONFIGURATION_REGISTERS: u64 = 0x01B4000D0;

const REGISTER_OFFSET: u64 = 0x08;

struct Timer {
    address: u64,
    duration: TickType,
    is_running: bool,
}

pub struct TimerBlock {
    pub timer0: Timer,
    pub timer1: Timer,
    pub timer2: Timer,
    pub timer3: Timer,
    pub timer4: Timer,
}

impl TimerBlock {
    pub fn new() -> Self {
        let timer0 = Timer::new(TIMER_0, 0x0);
        let timer1 = Timer::new(TIMER_1, 0x1);
        let timer2 = Timer::new(TIMER_2, 0x2);
        let timer3 = Timer::new(TIMER_3, 0x3);
        let timer4 = Timer::new(TIMER_4, 0x4);

        Self {
            timer0,
            timer1,
            timer2,
            timer3,
            timer4,
        }
    }
}

impl Timer {
    fn new(address: u64, enable_mask: u8) -> Self {
        let mut configuration_value: u8 = read_byte(CONFIGURATION_REGISTERS);
        configuration_value |= enable_mask;
        write_byte(CONFIGURATION_REGISTERS, configuration_value);

        Timer {
            address,
            duration: 0,
            is_running: false,
        }
    }
}

impl super::Timer for Timer {
    fn load_and_start(&self, value: TickType) {
        while read_byte(self.address + REGISTER_OFFSET) & 0x40 != 0 {
            // Wait for the previous load to finish
        }

        for i in 0..8 {
            write_byte(self.address + i, (value >> (i * 8)) & 0xFF)
        }
        self.is_running = true;
        self.duration = value;
    }

    fn now(&self) -> TickType {
        let mut control_value: u8 = read_byte(self.address + REGISTER_OFFSET);
        control_value |= 0x01;
        write_byte(self.address + REGISTER_OFFSET, control_value);

        while read_byte(self.address + REGISTER_OFFSET) & 0x20 != 0 {
            // Wait for the update to complete
        }

        let mut timer_value: TickType = 0x0;

        for i in 0..8 {
            timer_value |= (read_byte(self.address + i) as TickType) << (i * 8);
        }

        timer_value
    }

    unsafe fn read_byte(address: *const u8) -> u8 {
        *address
    }

    unsafe fn write_byte(address: *mut u8, value: u8) {
        *address = value
    }
}

/// Mips64 hardware timer setup.
pub fn setup_hardware_timer() {
    let timer_block = TimerBlock::new();

    let timer0 = timer_block.timer0;
    timer0.load_and_start(500 as TickType);

    unsafe {
        TIMER0 = Some(timer0);
    }
}

/// Mips64 getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    unsafe {
        let timer0 = TIMER0.take().expect("Timer error");
        let tick_counter = timer0.now();
        TIMER0 = Some(timer0);
        tick_counter.ticks()
    }
}
