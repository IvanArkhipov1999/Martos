use crate::timer::TickType;

/// Static variable for storing an instance of the timer block.
static mut TIMER_BLOCK: Option<TimerBlock> = None;

/// Base address of timer 0.
const TIMER_0: u64 = 0x01B400080;
/// Base address of timer 1.
const TIMER_1: u64 = 0x01B400090;
/// Base address of timer 2.
const TIMER_2: u64 = 0x01B4000A0;
/// Base address of timer 3.
const TIMER_3: u64 = 0x01B4000B0;
/// Base address of timer 4.
const TIMER_4: u64 = 0x01B4000C0;
/// Base address of configuration registers.
const CONFIGURATION_REGISTERS: u64 = 0x01B4000D0;

/// Offset for the status and control register.
const STATUS_AND_CONTROL_REGISTER_OFFSET: u64 = 0x08;

/// Structure representing a block of timers.
struct TimerBlock {
    /// Timer 0.
    timer0: Timer,
    /// Timer 1.
    timer1: Timer,
    /// Timer 2.
    timer2: Timer,
    /// Timer 3.
    timer3: Timer,
    /// Timer 4.
    timer4: Timer,
}

impl TimerBlock {
    /// Creates a new timer block and initializes each timer.
    fn new() -> Self {
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

/// Structure representing a timer.
struct Timer {
    /// Base address of timer.
    address: u64,
    /// The passed value for the counter.
    duration: TickType,
    /// An indicator showing whether the timer is running.
    is_running: bool,
}

impl Timer {
    /// Creates a new timer at the specified address and enables the timer counting.
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

    /// Loads a value into the timer, thereby starting it.
    fn load_and_start(&mut self, value: TickType) {
        while read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET) & 0x40 != 0 {
            // Wait for the previous load to finish
        }

        for i in 0..8 {
            write_byte(self.address + i, ((value >> (i * 8)) & 0xFF) as u8)
        }
        self.is_running = true;
        self.duration = value;
    }

    /// Gets the current value of the timer counter.
    fn now(&self) -> TickType {
        let mut control_value: u8 = read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET);
        control_value |= 0x01;
        write_byte(
            self.address + STATUS_AND_CONTROL_REGISTER_OFFSET,
            control_value,
        );

        while read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET) & 0x20 != 0 {
            // Wait for the update to complete
        }

        let mut counter_value: TickType = 0x0;
        for i in 0..8 {
            counter_value |= (read_byte(self.address + i) as TickType) << (i * 8);
        }

        counter_value
    }
}

/// Reads a byte from the given address.
fn read_byte(address: u64) -> u8 {
    unsafe { *(address as *const u8) }
}

/// Writes the given byte value to the specified address.
fn write_byte(address: u64, value: u8) {
    unsafe { *(address as *mut u8) = value }
}

/// Mips64 hardware timer setup.
pub fn setup_hardware_timer() {
    let mut timer_block = TimerBlock::new();

    timer_block.timer0.load_and_start(500 as TickType);

    unsafe {
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 getting hardware tick counter.
pub fn get_tick_counter() -> TickType {
    unsafe {
        let timer_block = TIMER_BLOCK.take().expect("Timer block error");
        let tick_counter = timer_block.timer0.now();
        TIMER_BLOCK = Some(timer_block);
        tick_counter
    }
}
