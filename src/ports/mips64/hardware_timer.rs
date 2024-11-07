use crate::timer::TickType;
use core::time::Duration;

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
/// Standard frequency of timer operation - 4 MHz.
const TIMER_FREQUENCY: u64 = 4;

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

/// Structure representing the timer.
struct Timer {
    /// Base address of the timer.
    address: u64,
    /// The passed value in ticks for the counter.
    duration: TickType,
    /// The count resolution mask for the timer.
    resolution_mask: u8,
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
            resolution_mask: enable_mask,
            is_running: false,
        }
    }

    /// Loads a value into the timer, thereby starting it.
    fn load_and_start(&mut self) {
        while read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET) & 0x40 != 0 {
            // Wait for the previous load to finish
        }

        if self.duration == 0 {
            return;
        }
        for i in 0..8 {
            write_byte(self.address + i, ((self.duration >> (i * 8)) & 0xFF) as u8)
        }
        self.is_running = true;
    }
  
    /// Changes the duration of the timer in the structure.
    fn change_duration(&mut self, ticks: TickType) {
        self.duration = ticks;
    }

    /// Gets the current ticks of the timer counter.
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

        let mut counter_ticks: TickType = 0x0;
        for i in 0..8 {
            counter_ticks |= (read_byte(self.address + i) as TickType) << (i * 8);
        }
      
        self.duration - counter_ticks
    }

    /// Disables the timer count.
    fn stop(&mut self) {
        let mut configuration_value: u8 = read_byte(CONFIGURATION_REGISTERS);
        configuration_value &= !self.resolution_mask;
        write_byte(CONFIGURATION_REGISTERS, configuration_value);
        self.is_running = false;
    }
}

/// Function to convert Duration to TickType. Return value will be saturated if exceed 64 bits.
fn duration_to_ticks(value: Duration) -> TickType {
    let micros = value.as_micros();
    let ticks = micros * TIMER_FREQUENCY as u128;

    if ticks > u64::MAX as u128 {
        u64::MAX as TickType
    } else {
        ticks as TickType
    }
}

/// Function to convert TickType to Duration.
fn ticks_to_duration(ticks: TickType) -> Duration {
    let micros = ticks / TIMER_FREQUENCY;

    Duration::from_micros(micros)
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
    let timer_block = TimerBlock::new();

    unsafe {
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 start harware timer.
pub fn start_hardware_timer() {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        timer_block.timer0.load_and_start();
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 change the period of a timer.
/// If timer was in active state, function will restart timer with a new period.
pub fn change_period_timer(period: Duration) {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        timer_block
            .timer0
            .change_duration(duration_to_ticks(period));
        if timer_block.timer0.is_running {
            timer_block.timer0.load_and_start();
        }
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 getting counter value.
pub fn get_time() -> Duration {
    unsafe {
        let timer_block = TIMER_BLOCK.take().expect("Timer block error");
        let tick_counter = timer_block.timer0.now();
        TIMER_BLOCK = Some(timer_block);

        ticks_to_duration(tick_counter)
    }
}

/// Mips64 stop hardware timer.
pub fn stop_hardware_timer() -> bool {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        timer_block.timer0.stop();
        TIMER_BLOCK = Some(timer_block);
    }

    true
}
