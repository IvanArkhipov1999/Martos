use crate::timer::TickType;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

// Declare timer_tests file as child file to test private functions.
#[cfg(test)]
#[path = "../../../tests/mips64/timer_tests.rs"]
mod mips64_timer_tests;

/// Static variable for storing an instance of the timer block.
static mut TIMER_BLOCK: Option<TimerBlock<MemoryAccess>> = None;

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
struct TimerBlock<M: ByteAccess> {
    /// Timer 0.
    timer0: Timer<M>,
    /// Timer 1.
    timer1: Timer<M>,
    /// Timer 2.
    timer2: Timer<M>,
    /// Timer 3.
    timer3: Timer<M>,
    /// Timer 4.
    timer4: Timer<M>,
}

impl<M: ByteAccess + core::clone::Clone> TimerBlock<M> {
    /// Creates a new timer block and initializes each timer.
    fn new(accessibility: M) -> Self {
        let timer0 = Timer::new(TIMER_0, 0x1, accessibility.clone());
        let timer1 = Timer::new(TIMER_1, 0x2, accessibility.clone());
        let timer2 = Timer::new(TIMER_2, 0x4, accessibility.clone());
        let timer3 = Timer::new(TIMER_3, 0x8, accessibility.clone());
        let timer4 = Timer::new(TIMER_4, 0x10, accessibility.clone());

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
struct Timer<M: ByteAccess> {
    /// Base address of the timer.
    address: u64,

    /// The passed value in ticks for the counter.
    duration: TickType,

    /// The count resolution mask for the timer.
    resolution_mask: u8,

    /// An indicator showing whether the timer is in use.
    in_use: AtomicBool,

    /// An indicator showing whether the timer is in auto reload mode or in one shot mode.
    reload_mode: bool,

    /// An indicator showing whether it is possible to start loading a new value into the timer.
    lock_for_load: AtomicBool,

    /// An indicator showing whether it is possible to start receiving the current ticks of the timer counter.
    lock_for_now: AtomicBool,

    /// Methods for reading and writing a byte at a given address.
    accessibility: M,
}

impl<M: ByteAccess> Timer<M> {
    /// Creates a new timer at the specified address.
    fn new(address: u64, enable_mask: u8, accessibility: M) -> Self {
        Timer {
            address,
            duration: 0,
            resolution_mask: enable_mask,
            in_use: AtomicBool::new(false),
            reload_mode: false,
            lock_for_load: AtomicBool::new(false),
            lock_for_now: AtomicBool::new(false),
            accessibility,
        }
    }

    /// Enables the timer counting.
    fn start(&mut self) {
        if self.duration == 0 {
            return;
        }

        let mut configuration_value: u8 = self.accessibility.read_byte(CONFIGURATION_REGISTERS);
        configuration_value |= self.resolution_mask;
        self.accessibility
            .write_byte(CONFIGURATION_REGISTERS, configuration_value);
    }

    /// Changes the timer's operating mode.
    fn change_operating_mode(&mut self, auto_reload: bool) {
        if self.reload_mode == auto_reload {
            return;
        }

        let mut control_value: u8 = self
            .accessibility
            .read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET);
        if auto_reload {
            control_value |= 0x04;
        } else {
            control_value &= 0xfb;
        }
        self.accessibility.write_byte(
            self.address + STATUS_AND_CONTROL_REGISTER_OFFSET,
            control_value,
        );
        self.reload_mode = auto_reload;
    }

    /// Loads a value into the timer.
    fn load_value(&mut self, ticks: TickType) {
        while self.lock_for_load.swap(true, Ordering::Acquire) {
            // Wait until lock_for_load becomes false
        }

        while self
            .accessibility
            .read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET)
            & 0x40
            != 0
        {
            // Wait for the previous load to finish
        }

        for i in 0..8 {
            self.accessibility
                .write_byte(self.address + i, ((ticks >> (i * 8)) & 0xFF) as u8)
        }
        self.duration = ticks;

        self.lock_for_load.store(false, Ordering::Release);
    }

    /// Gets the current ticks of the timer counter.
    fn now(&self) -> TickType {
        while self.lock_for_now.swap(true, Ordering::Acquire) {
            // Wait until lock_for_now becomes false
        }

        let mut control_value: u8 = self
            .accessibility
            .read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET);
        control_value |= 0x01;
        self.accessibility.write_byte(
            self.address + STATUS_AND_CONTROL_REGISTER_OFFSET,
            control_value,
        );

        while self
            .accessibility
            .read_byte(self.address + STATUS_AND_CONTROL_REGISTER_OFFSET)
            & 0x20
            != 0
        {
            // Wait for the update to complete
        }

        let mut counter_ticks: TickType = 0x0;
        for i in 0..8 {
            counter_ticks |=
                (self.accessibility.read_byte(self.address + i) as TickType) << (i * 8);
        }

        self.lock_for_now.store(false, Ordering::Release);

        self.duration - counter_ticks
    }

    /// Disables the timer count.
    fn stop(&mut self) {
        let mut configuration_value: u8 = self.accessibility.read_byte(CONFIGURATION_REGISTERS);
        configuration_value &= !self.resolution_mask;
        self.accessibility
            .write_byte(CONFIGURATION_REGISTERS, configuration_value);
    }
}

/// Function to convert Duration to TickType.
/// Return value will be saturated if exceed 64 bits.
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

/// Provides methods for reading and writing individual bytes at a given address.
/// This trait is required to implement both direct memory access and simulated memory access for tests.
trait ByteAccess {
    /// Reads a byte from the given address.
    fn read_byte(&self, address: u64) -> u8;

    /// Writes the given byte value to the specified address.
    fn write_byte(&self, address: u64, value: u8);
}

/// Provides the ability to access bytes in memory.
#[derive(Clone)]
struct MemoryAccess;
impl ByteAccess for MemoryAccess {
    fn read_byte(&self, address: u64) -> u8 {
        unsafe { *(address as *const u8) }
    }

    fn write_byte(&self, address: u64, value: u8) {
        unsafe { *(address as *mut u8) = value };
    }
}

/// Mips64 hardware timer setup.
pub fn setup_hardware_timer() {
    let timer_block = TimerBlock::new(MemoryAccess);

    unsafe {
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 attempt to acquire timer.
pub fn try_acquire_timer(timer_index: u8) -> bool {
    if timer_index <= 4 as u8 {
        unsafe {
            let timer_block = TIMER_BLOCK.take().expect("Timer block error");

            let timers = [
                &timer_block.timer0.in_use,
                &timer_block.timer1.in_use,
                &timer_block.timer2.in_use,
                &timer_block.timer3.in_use,
                &timer_block.timer4.in_use,
            ];

            let return_value = match timers[timer_index as usize].compare_exchange(
                false,
                true,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => true,
                Err(_) => false,
            };
            TIMER_BLOCK = Some(timer_block);

            return_value
        }
    } else {
        false
    }
}

/// Mips64 start harware timer.
pub fn start_hardware_timer(timer_index: u8) {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        match timer_index {
            0 => timer_block.timer0.start(),
            1 => timer_block.timer1.start(),
            2 => timer_block.timer2.start(),
            3 => timer_block.timer3.start(),
            4 => timer_block.timer4.start(),
            _ => (),
        }
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 change operating mode of hardware timer.
pub fn set_reload_mode(timer_index: u8, auto_reload: bool) {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        match timer_index {
            0 => timer_block.timer0.change_operating_mode(auto_reload),
            1 => timer_block.timer1.change_operating_mode(auto_reload),
            2 => timer_block.timer2.change_operating_mode(auto_reload),
            3 => timer_block.timer3.change_operating_mode(auto_reload),
            4 => timer_block.timer4.change_operating_mode(auto_reload),
            _ => (),
        }
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 change the period of hardware timer.
/// If timer was in active state, function will restart timer with a new period.
pub fn change_period_timer(timer_index: u8, period: Duration) {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        match timer_index {
            0 => timer_block.timer0.load_value(duration_to_ticks(period)),
            1 => timer_block.timer1.load_value(duration_to_ticks(period)),
            2 => timer_block.timer2.load_value(duration_to_ticks(period)),
            3 => timer_block.timer3.load_value(duration_to_ticks(period)),
            4 => timer_block.timer4.load_value(duration_to_ticks(period)),
            _ => (),
        }
        TIMER_BLOCK = Some(timer_block);
    }
}

/// Mips64 getting counter value of hardware timer.
pub fn get_time(timer_index: u8) -> Duration {
    unsafe {
        let timer_block = TIMER_BLOCK.take().expect("Timer block error");
        let tick_counter = match timer_index {
            0 => timer_block.timer0.now(),
            1 => timer_block.timer1.now(),
            2 => timer_block.timer2.now(),
            3 => timer_block.timer3.now(),
            4 => timer_block.timer4.now(),
            _ => 0,
        };
        TIMER_BLOCK = Some(timer_block);

        ticks_to_duration(tick_counter)
    }
}

/// Mips64 stop hardware timer.
pub fn stop_hardware_timer(timer_index: u8) -> bool {
    unsafe {
        let mut timer_block = TIMER_BLOCK.take().expect("Timer block error");
        match timer_index {
            0 => timer_block.timer0.stop(),
            1 => timer_block.timer1.stop(),
            2 => timer_block.timer2.stop(),
            3 => timer_block.timer3.stop(),
            4 => timer_block.timer4.stop(),
            _ => (),
        }
        TIMER_BLOCK = Some(timer_block);
    }

    true
}

/// Mips64 release hardware timer.
pub fn release_hardware_timer(timer_index: u8) {
    unsafe {
        let timer_block = TIMER_BLOCK.take().expect("Timer block error");
        match timer_index {
            0 => timer_block.timer0.in_use.store(false, Ordering::Release),
            1 => timer_block.timer1.in_use.store(false, Ordering::Release),
            2 => timer_block.timer2.in_use.store(false, Ordering::Release),
            3 => timer_block.timer3.in_use.store(false, Ordering::Release),
            4 => timer_block.timer4.in_use.store(false, Ordering::Release),
            _ => (),
        }
        TIMER_BLOCK = Some(timer_block);
    }
}
