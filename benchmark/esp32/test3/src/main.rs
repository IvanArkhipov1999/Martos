#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use esp_backtrace as _;
use esp_hal::entry;
use esp_hal::xtensa_lx_rt::xtensa_lx::timer::delay;
use esp_println::println;
use martos::{
    init_system,
    task_manager::{TaskManager, TaskManagerTrait},
    timer::Timer,
};

/// Счётчики для 5 задач - каждая задача инкрементирует свой счётчик на 1000 в цикле
/// Все задачи работают в preemptive планировщике с одинаковым приоритетом (round-robin)
static COUNTER1: AtomicU32 = AtomicU32::new(0);
static COUNTER2: AtomicU32 = AtomicU32::new(0);
static COUNTER3: AtomicU32 = AtomicU32::new(0);
static COUNTER4: AtomicU32 = AtomicU32::new(0);
static COUNTER5: AtomicU32 = AtomicU32::new(0);

/// Setup функция для задачи счётчика 1
fn counter1_task_setup_fn() {
    println!("Counter1 task setup - будет делать 1000 итерации, прибавляя по 1");
}

/// Loop функция для задачи счётчика 1 - делает 1000 итерации, прибавляя по 1
fn counter1_task_loop_fn() {
    for _ in 0..1000 {
        COUNTER1.fetch_add(1, Ordering::Relaxed);
    }
    // Небольшая задержка для preemptive планировщика
    // delay(1000);
    println!("Counter1");
}

/// Stop condition для задачи счётчика 1
fn counter1_task_stop_condition_fn() -> bool {
    COUNTER1.load(Ordering::Relaxed) >= 100000
}

/// Setup функция для задачи счётчика 2
fn counter2_task_setup_fn() {
    println!("Counter2 task setup - будет делать 1000 итерации, прибавляя по 1");
}

/// Loop функция для задачи счётчика 2 - делает 1000 итерации, прибавляя по 1
fn counter2_task_loop_fn() {
    for _ in 0..1000 {
        COUNTER2.fetch_add(1, Ordering::Relaxed);
    }
    // Небольшая задержка для preemptive планировщика
    // delay(1000);
    println!("Counter2");
}

/// Stop condition для задачи счётчика 2
fn counter2_task_stop_condition_fn() -> bool {
    COUNTER2.load(Ordering::Relaxed) >= 100000
}

/// Setup функция для задачи счётчика 3
fn counter3_task_setup_fn() {
    println!("Counter3 task setup - будет делать 1000 итерации, прибавляя по 1");
}

/// Loop функция для задачи счётчика 3 - делает 1000 итерации, прибавляя по 1
fn counter3_task_loop_fn() {
    for _ in 0..1000 {
        COUNTER3.fetch_add(1, Ordering::Relaxed);
    }
    // Небольшая задержка для preemptive планировщика
    // delay(1000);
    println!("Counter3");
}

/// Stop condition для задачи счётчика 3
fn counter3_task_stop_condition_fn() -> bool {
    COUNTER3.load(Ordering::Relaxed) >= 100000
}

/// Setup функция для задачи счётчика 4
fn counter4_task_setup_fn() {
    println!("Counter4 task setup - будет делать 1000 итерации, прибавляя по 1");
}

/// Loop функция для задачи счётчика 4 - делает 1000 итерации, прибавляя по 1
fn counter4_task_loop_fn() {
    for _ in 0..1000 {
        COUNTER4.fetch_add(1, Ordering::Relaxed);
    }
    // Небольшая задержка для preemptive планировщика
    // delay(1000);
    println!("Counter4");
}

/// Stop condition для задачи счётчика 4
fn counter4_task_stop_condition_fn() -> bool {
    COUNTER4.load(Ordering::Relaxed) >= 100000
}

/// Setup функция для задачи счётчика 5
fn counter5_task_setup_fn() {
    println!("Counter5 task setup - будет делать 1000 итерации, прибавляя по 1");
}

/// Loop функция для задачи счётчика 5 - делает 1000 итерации, прибавляя по 1
fn counter5_task_loop_fn() {
    for _ in 0..1000 {
        COUNTER5.fetch_add(1, Ordering::Relaxed);
    }
    // Небольшая задержка для preemptive планировщика
    // delay(1000);
    println!("Counter5");
}

/// Stop condition для задачи счётчика 5
fn counter5_task_stop_condition_fn() -> bool {
    COUNTER5.load(Ordering::Relaxed) >= 100000
}

/// Setup функция для задачи мониторинга
fn monitor_task_setup_fn() {
    println!("Monitor task setup - будет печатать значения аппаратного таймера и всех счётчиков");
    
    // Инициализируем Timer 1 для измерения времени
    if let Some(timer1) = Timer::get_timer(1) {
        timer1.set_reload_mode(true);  // Периодический режим
        timer1.change_period_timer(core::time::Duration::from_millis(1));  // 1 мс период
        timer1.start_timer();
        timer1.release_timer();
        println!("Timer 1 initialized for time measurement");
    } else {
        println!("Failed to acquire Timer 1 for time measurement");
    }
}

/// Loop функция для задачи мониторинга - печатает значение аппаратного таймера и всех счётчиков
fn monitor_task_loop_fn() {
    // Используем Timer 1 (TIMER10) для измерения времени, так как Timer 0 занят планировщиком
    if let Some(timer1) = Timer::get_timer(1) {
        let time = timer1.get_time();
        timer1.release_timer();
        
        let c1 = COUNTER1.load(Ordering::Relaxed);
        let c2 = COUNTER2.load(Ordering::Relaxed);
        let c3 = COUNTER3.load(Ordering::Relaxed);
        let c4 = COUNTER4.load(Ordering::Relaxed);
        let c5 = COUNTER5.load(Ordering::Relaxed);
        
        println!("Monitor: time = {}s {}μs, counters = [{}, {}, {}, {}, {}]", 
                time.as_secs(), time.subsec_micros(), c1, c2, c3, c4, c5);
    } else {
        println!("Monitor task: timer 1 is busy");
    }
    // Задержка для preemptive планировщика
    // delay(10000);
}

/// Stop condition для задачи мониторинга - никогда не останавливается
fn monitor_task_stop_condition_fn() -> bool {
    false
}

#[entry]
fn main() -> ! {
    // Инициализация Martos
    init_system();
    
    // Добавляем 5 задач счётчиков
    TaskManager::add_task(
        counter1_task_setup_fn,
        counter1_task_loop_fn,
        counter1_task_stop_condition_fn,
    );
    
    TaskManager::add_task(
        counter2_task_setup_fn,
        counter2_task_loop_fn,
        counter2_task_stop_condition_fn,
    );
    
    TaskManager::add_task(
        counter3_task_setup_fn,
        counter3_task_loop_fn,
        counter3_task_stop_condition_fn,
    );
    
    TaskManager::add_task(
        counter4_task_setup_fn,
        counter4_task_loop_fn,
        counter4_task_stop_condition_fn,
    );
    
    TaskManager::add_task(
        counter5_task_setup_fn,
        counter5_task_loop_fn,
        counter5_task_stop_condition_fn,
    );
    
    // Добавляем задачу мониторинга (печатает таймер и счётчики)
    TaskManager::add_task(
        monitor_task_setup_fn,
        monitor_task_loop_fn,
        monitor_task_stop_condition_fn,
    );
    
    // Запускаем preemptive планировщик задач (round-robin с одинаковым приоритетом)
    TaskManager::start_task_manager();
}
