#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use esp_backtrace as _;
use esp_hal::entry;
use esp_println::println;
use martos::{
    init_system,
    task_manager::{TaskManager, TaskManagerTrait},
    timer::Timer,
};

/// Counter для первой задачи - инкрементируется по 1 в цикле из 1000 итераций
static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Setup функция для задачи счётчика
fn counter_task_setup_fn() {
    println!("Counter task setup - будет делать 1000 итерации, прибавляя по 1");
}

/// Loop функция для задачи счётчика - делает 1000 итерации, прибавляя по 1
fn counter_task_loop_fn() {
    for _ in 0..1000 {
        let current = COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    let current = COUNTER.load(Ordering::Relaxed);
    println!("Counter task: value = {}", current);
}

/// Stop condition для задачи счётчика
fn counter_task_stop_condition_fn() -> bool {
    let current = COUNTER.load(Ordering::Relaxed);
    current >= 100000 // Остановим после достаточного количества итераций
}

/// Setup функция для задачи таймера
fn timer_task_setup_fn() {
    println!("Timer task setup - будет печатать значение аппаратного таймера");
}

/// Loop функция для задачи таймера - печатает значение аппаратного таймера
fn timer_task_loop_fn() {
    if let Some(timer0) = Timer::get_timer(0) {
        let time = timer0.get_time();
        timer0.release_timer();
        println!("Timer task: time = {} seconds, {} microseconds", 
                time.as_secs(), time.subsec_micros());
    } else {
        println!("Timer task: timer 0 is busy");
    }
}

/// Stop condition для задачи таймера - никогда не останавливается
fn timer_task_stop_condition_fn() -> bool {
    false
}

#[entry]
fn main() -> ! {
    // Инициализация Martos
    init_system();
    
    // Добавляем задачу счётчика
    TaskManager::add_task(
        counter_task_setup_fn,
        counter_task_loop_fn,
        counter_task_stop_condition_fn,
    );
    
    // Добавляем задачу таймера
    TaskManager::add_task(
        timer_task_setup_fn,
        timer_task_loop_fn,
        timer_task_stop_condition_fn,
    );
    
    // Запускаем планировщик задач
    TaskManager::start_task_manager();
}
