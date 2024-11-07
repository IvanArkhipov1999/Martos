use crate::ports::xtensa_esp32::hardware_timer::TIMER00;
use esp_hal::timer::timg::{Timer, Timer0, TimerGroup};
use esp_hal::trapframe::TrapFrame;
use esp_hal::xtensa_lx_rt;
use esp_hal::{
    interrupt::{self, InterruptHandler, Priority},
    prelude::*,
};
use esp_hal::{peripherals::*, prelude::*, Cpu};
use esp_println::println;

pub fn setup_interrupt() {
    println!("Setup interrupt");

    let timer0 = unsafe { TIMER00.take().expect("Timer error") };
    // timer0.set_interrupt_handler(tg0_t0_level);
    timer0.set_interrupt_handler(InterruptHandler::new(
        unsafe { core::mem::transmute::<*const (), extern "C" fn()>(handler as *const ()) },
        interrupt::Priority::Priority1,
    ));
    timer0.enable_interrupt(true);
    timer0.enable_auto_reload(true);
    interrupt::enable(Interrupt::TG0_T0_LEVEL, Priority::Priority1).unwrap();

    // timer0.reset();
    timer0.load_value(1000u64.millis()).unwrap();
    timer0.start();
    timer0.listen();

    unsafe {
        TIMER00 = Some(timer0);
    };
}

extern "C" fn handler(ctx: &mut TrapFrame) {
    // todo: should disable interrupts?
    println!("Handler\nCTX: {:?}", ctx);

    let mut timer00 = unsafe { TIMER00.take().expect("Timer error") };
    timer00.clear_interrupt();
    unsafe {
        TIMER00 = Some(timer00);
    };

    crate::task_manager::tm::TM::schedule(ctx);
}

pub fn setup_stack(thread: &mut crate::task_manager::tm::Thread) {
    // todo!("setup SP, PC(fn pointer needed), whatever else is needed")
    thread.context.PC = thread.func as u32;
    thread.context.A0 = 0; // return address

    let stack_ptr = thread.stack as usize + crate::task_manager::tm::THREAD_STACK_SIZE; // stack pointer todo: +stack_size?
    thread.context.A1 = stack_ptr as u32;
}

pub fn save_ctx(thread_ctx: &mut TrapFrame, isr_ctx: &TrapFrame) {
    *thread_ctx = *isr_ctx
}

pub fn load_ctx(thread_ctx: &TrapFrame, isr_ctx: &mut TrapFrame) {
    *isr_ctx = *thread_ctx
}
