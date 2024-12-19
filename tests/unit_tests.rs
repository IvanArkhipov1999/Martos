// #[cfg(all(test, not(feature = "mips64_timer_tests")))]
mod unit_tests {
    use martos::task_manager::{TaskManager, TaskManagerTrait};
    use martos::timer::Timer;
    use sequential_test::sequential;
    use std::cell::RefCell;
    use std::{
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };
    // TODO: refactor unit tests. They should check less. Separate tests for setup, loop and stop functions.
    // TODO: refactor unit tests. Task manager and timer tests should be in different files in one directory.

    fn dummy_setup_fn() {}
    fn dummy_loop_fn() {}
    fn dummy_condition_true() -> bool {
        true
    }

    #[test]
    #[sequential]
    #[should_panic(
        expected = "Error: add_task: Task's priority is invalid. It must be between 0 and 11."
    )]
    /// Test add a task with nonexistent priority
    fn test_add_task_invalid_priority() {
        TaskManager::reset_task_manager();
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 100);
        TaskManager::test_start_task_manager();
    }

    #[test]
    #[sequential]
    fn test_add_two_priority_tasks_and_check_vectors() {
        TaskManager::reset_task_manager();
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 0);
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 1);
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 0);

        assert_eq!(TaskManager::count_tasks_with_priority(0), 2);
        assert_eq!(TaskManager::count_tasks_with_priority(1), 1);
        assert_eq!(TaskManager::count_all_tasks(), 3);
    }

    #[test]
    #[sequential]
    fn test_add_task_and_check_if_priority_zero() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true);
        assert_eq!(TaskManager::count_tasks_with_priority(0), 1);
        assert_eq!(TaskManager::count_all_tasks(), 1);
    }

    #[test]
    #[sequential]
    fn test_get_task_by_id() {
        TaskManager::reset_task_manager();
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 10);
        let id = TaskManager::get_id_from_position(10, 0);

        let found_task = TaskManager::get_task_from_id(id);

        assert_eq!(id, TaskManager::get_id_from_task(found_task));
    }

    #[test]
    #[sequential]
    #[should_panic(expected = "Error: get_task_from_id: Task with this id not found.")]
    fn test_get_task_by_invalid_id() {
        TaskManager::reset_task_manager();
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 10);
        let found_task = TaskManager::get_task_from_id(2);
    }

    #[test]
    #[sequential]
    fn test_get_id_from_position() {
        TaskManager::reset_task_manager();
        // ID of a first added task is 1.
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 10);

        let task_id = TaskManager::get_id_from_position(10, 0);
        assert_eq!(task_id, 1);
    }

    #[test]
    #[sequential]
    #[should_panic(expected = "Error: get_id_from_position: Position out of bounds.")]
    fn test_get_id_from_invalid_position() {
        TaskManager::reset_task_manager();
        // ID of a first added task is 1.
        TaskManager::add_priority_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true, 10);
        let task_id = TaskManager::get_id_from_position(10, 1);
    }

    #[test]
    #[sequential]
    fn test_get_id_from_task() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true);

        let task = TaskManager::get_task_from_id(1);
        assert_eq!(TaskManager::get_id_from_task(task), 1);
    }

    fn test_put_to_sleep_loop_fn() {
        TaskManager::put_to_sleep(1);
    }

    #[test]
    #[sequential]
    #[should_panic(expected = "Error: put_to_sleep: Task with this id is currently running.")]
    fn test_put_to_sleep_running_task() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(
            dummy_setup_fn,
            test_put_to_sleep_loop_fn,
            dummy_condition_true,
        );
        TaskManager::test_start_task_manager();
    }

    #[test]
    #[sequential]
    #[should_panic(expected = "Error: put_to_sleep: Task with this id is currently sleeping.")]
    fn test_put_to_sleep_sleeping_task() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true);

        let task_1 = TaskManager::get_task_from_id(1);

        assert_eq!(TaskManager::get_status(task_1), TaskManager::ready_status());

        TaskManager::add_task(
            dummy_setup_fn,
            test_put_to_sleep_loop_fn,
            dummy_condition_true,
        );
        TaskManager::schedule();
        assert_eq!(
            TaskManager::get_status(task_1),
            TaskManager::sleeping_status()
        );

        TaskManager::add_task(
            dummy_setup_fn,
            test_put_to_sleep_loop_fn,
            dummy_condition_true,
        );
        TaskManager::test_start_task_manager();
    }

    #[test]
    #[sequential]
    #[should_panic(
        expected = "Error: put_to_sleep: Task with this id is terminated and soon will be removed."
    )]
    fn test_put_to_sleep_terminated_task() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true);

        let task_1 = TaskManager::get_task_from_id(1);

        assert_eq!(TaskManager::get_status(task_1), TaskManager::ready_status());

        TaskManager::schedule();

        assert_eq!(
            TaskManager::get_status(task_1),
            TaskManager::terminated_status()
        );

        TaskManager::add_task(
            dummy_setup_fn,
            test_put_to_sleep_loop_fn,
            dummy_condition_true,
        );

        TaskManager::test_start_task_manager();
    }

    #[test]
    #[sequential]
    fn test_put_to_sleep_task_from_task() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(dummy_setup_fn, dummy_loop_fn, dummy_condition_true);
        assert_eq!(TaskManager::get_id_from_position(0, 0), 1);

        TaskManager::add_task(
            dummy_setup_fn,
            test_put_to_sleep_loop_fn,
            dummy_condition_true,
        );
        assert_eq!(TaskManager::get_id_from_position(0, 1), 2);

        let task_1 = TaskManager::get_task_from_id(1);

        assert_eq!(TaskManager::get_status(task_1), TaskManager::ready_status());

        TaskManager::test_start_task_manager();

        assert_eq!(
            TaskManager::get_status(task_1),
            TaskManager::sleeping_status()
        );
    }

    thread_local! {
        static EXEC_ORDER: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
    }
    #[test]
    #[sequential]
    fn test_get_next_task_same_priority() {
        TaskManager::reset_task_manager();
        fn first_task_loop_fn() {
            EXEC_ORDER.with(|order| {
                order.borrow_mut().push("first");
            });
        }

        fn second_task_loop_fn() {
            EXEC_ORDER.with(|order| {
                order.borrow_mut().push("second");
            });
        }

        fn third_task_loop_fn() {
            EXEC_ORDER.with(|order| {
                order.borrow_mut().push("third");
            });
        }
        TaskManager::add_task(dummy_setup_fn, first_task_loop_fn, dummy_condition_true);
        TaskManager::add_task(dummy_setup_fn, second_task_loop_fn, dummy_condition_true);
        TaskManager::add_task(dummy_setup_fn, third_task_loop_fn, dummy_condition_true);

        TaskManager::test_start_task_manager();

        EXEC_ORDER.with(|order| {
            assert_eq!(*order.borrow(), ["third", "second", "first"]);
        });
    }

    #[test]
    #[sequential]
    /// Tests if task manager without tasks works during some time.
    fn test_empty_task_manager() {
        TaskManager::reset_task_manager();
        TaskManager::test_start_task_manager();
        assert!(TaskManager::has_no_tasks());
    }

    /// Counter for a task for test_one_finite_task_task_manager.
    static TEST_ONE_FINITE_TASK_TASK_MANAGER_COUNTER: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_one_finite_task_task_manager.
    fn test_one_finite_task_task_manager_setup_fn() {}
    /// Loop function for task for test_one_finite_task_task_manager.
    fn test_one_finite_task_task_manager_loop_fn() {
        TEST_ONE_FINITE_TASK_TASK_MANAGER_COUNTER.fetch_add(1, Ordering::Relaxed);
        // То есть состояния помогают только контролировать flow????
        // А по факту вся суть в состояниях... или наоборот, состояния формируют работу, а эти все функции для проверки чисто рудименты...
    }
    /// Stop function for task for test_one_finite_task_task_manager.
    fn test_one_finite_task_task_manager_stop_condition_fn() -> bool {
        let value = unsafe { TEST_ONE_FINITE_TASK_TASK_MANAGER_COUNTER.as_ptr().read() };
        if value % 50 == 0 {
            return true;
        }
        false
    }
    #[test]
    #[sequential]
    /// Tests if task manager with one finite task works correctly during some time without panic.
    fn test_one_finite_task_task_manager() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(
            test_one_finite_task_task_manager_setup_fn,
            test_one_finite_task_task_manager_loop_fn,
            test_one_finite_task_task_manager_stop_condition_fn,
        );
        TaskManager::test_start_task_manager();

        assert_eq!(
            unsafe { TEST_ONE_FINITE_TASK_TASK_MANAGER_COUNTER.as_ptr().read() },
            50
        );
    }

    /// Counter for a task for test_one_infinite_task_task_manager.
    static TEST_ONE_INFINITE_TASK_TASK_MANAGER_COUNTER: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_one_infinite_task_task_manager.
    fn test_one_infinite_task_task_manager_setup_fn() {}
    /// Loop function for task for test_one_infinite_task_task_manager.
    fn test_one_infinite_task_task_manager_loop_fn() {
        TEST_ONE_INFINITE_TASK_TASK_MANAGER_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for task for test_one_infinite_task_task_manager.
    fn test_one_infinite_task_task_manager_stop_condition_fn() -> bool {
        false
    }
    #[test]
    #[sequential]
    /// Tests if task manager with one infinite task works correctly during some time without panic.
    fn test_one_infinite_task_task_manager() {
        TaskManager::add_task(
            test_one_infinite_task_task_manager_setup_fn,
            test_one_infinite_task_task_manager_loop_fn,
            test_one_infinite_task_task_manager_stop_condition_fn,
        );
        TaskManager::test_start_task_manager();
        TaskManager::reset_task_manager();
    }

    /// Counter for a task for test_two_finite_tasks_task_manager.
    static TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER1: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_two_finite_tasks_task_manager.
    fn test_two_finite_tasks_task_manager_setup_fn1() {}
    /// Loop function for task for test_two_finite_tasks_task_manager.
    fn test_two_finite_tasks_task_manager_loop_fn1() {
        TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER1.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for task for test_two_finite_tasks_task_manager.
    fn test_two_finite_tasks_task_manager_stop_condition_fn1() -> bool {
        let value = unsafe { TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER1.as_ptr().read() };
        if value % 50 == 0 {
            return true;
        }
        false
    }
    /// Counter for a task for test_two_finite_tasks_task_manager.
    static TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER2: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_two_finite_tasks_task_manager.
    fn test_two_finite_tasks_task_manager_setup_fn2() {}
    /// Loop function for task for test_two_finite_tasks_task_manager.
    fn test_two_finite_tasks_task_manager_loop_fn2() {
        TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER2.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for task for test_two_finite_tasks_task_manager.
    fn test_two_finite_tasks_task_manager_stop_condition_fn2() -> bool {
        let value = unsafe { TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER2.as_ptr().read() };
        if value % 25 == 0 {
            return true;
        }
        false
    }
    #[test]
    #[sequential]
    /// Tests if task manager with two finite tasks works correctly during some time without panic.
    fn test_two_finite_tasks_task_manager() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(
            test_two_finite_tasks_task_manager_setup_fn1,
            test_two_finite_tasks_task_manager_loop_fn1,
            test_two_finite_tasks_task_manager_stop_condition_fn1,
        );
        TaskManager::add_task(
            test_two_finite_tasks_task_manager_setup_fn2,
            test_two_finite_tasks_task_manager_loop_fn2,
            test_two_finite_tasks_task_manager_stop_condition_fn2,
        );
        TaskManager::test_start_task_manager();

        assert_eq!(
            unsafe { TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER1.as_ptr().read() },
            50
        );
        assert_eq!(
            unsafe { TEST_TWO_FINITE_TASK_TASK_MANAGER_COUNTER2.as_ptr().read() },
            25
        );
    }

    /// Counter for a task for test_two_different_tasks_task_manager.
    static TEST_TWO_DIFFERENT_TASK_TASK_MANAGER_COUNTER1: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_two_different_tasks_task_manager.
    fn test_two_different_tasks_task_manager_setup_fn1() {}
    /// Loop function for task for test_two_different_tasks_task_manager.
    fn test_two_different_tasks_task_manager_loop_fn1() {
        TEST_TWO_DIFFERENT_TASK_TASK_MANAGER_COUNTER1.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for a task for test_two_different_tasks_task_manager.
    fn test_two_different_tasks_task_manager_stop_condition_fn1() -> bool {
        let value = unsafe {
            TEST_TWO_DIFFERENT_TASK_TASK_MANAGER_COUNTER1
                .as_ptr()
                .read()
        };
        if value % 50 == 0 {
            return true;
        }
        false
    }
    /// Counter for a task for test_two_different_tasks_task_manager.
    static TEST_TWO_DIFFERENT_TASK_TASK_MANAGER_COUNTER2: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_two_different_tasks_task_manager.
    fn test_two_different_tasks_task_manager_setup_fn2() {}
    /// Loop function for task for test_two_different_tasks_task_manager.
    fn test_two_different_tasks_task_manager_loop_fn2() {
        TEST_TWO_DIFFERENT_TASK_TASK_MANAGER_COUNTER2.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for a task for test_two_different_tasks_task_manager.
    fn test_two_different_tasks_task_manager_stop_condition_fn2() -> bool {
        false
    }
    #[test]
    #[sequential]
    /// Tests if task manager with two different (finite and infinite) tasks works correctly during some time without panic.
    fn test_two_different_tasks_task_manager() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(
            test_two_different_tasks_task_manager_setup_fn1,
            test_two_different_tasks_task_manager_loop_fn1,
            test_two_different_tasks_task_manager_stop_condition_fn1,
        );
        TaskManager::add_task(
            test_two_different_tasks_task_manager_setup_fn2,
            test_two_different_tasks_task_manager_loop_fn2,
            test_two_different_tasks_task_manager_stop_condition_fn2,
        );
        TaskManager::test_start_task_manager();

        assert_eq!(
            unsafe {
                TEST_TWO_DIFFERENT_TASK_TASK_MANAGER_COUNTER1
                    .as_ptr()
                    .read()
            },
            50
        );
    }

    /// Counter for a task for test_two_infinite_tasks_task_manager.
    static TEST_TWO_INFINITE_TASK_TASK_MANAGER_COUNTER1: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_two_infinite_tasks_task_manager.
    fn test_two_infinite_tasks_task_manager_setup_fn1() {}
    /// Loop function for task for test_two_infinite_tasks_task_manager.
    fn test_two_infinite_tasks_task_manager_loop_fn1() {
        TEST_TWO_INFINITE_TASK_TASK_MANAGER_COUNTER1.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for a task for test_two_infinite_tasks_task_manager.
    fn test_two_infinite_tasks_task_manager_stop_condition_fn1() -> bool {
        false
    }
    /// Counter for a task for test_two_infinite_tasks_task_manager.
    static TEST_TWO_INFINITE_TASK_TASK_MANAGER_COUNTER2: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_two_infinite_tasks_task_manager.
    fn test_two_infinite_tasks_task_manager_setup_fn2() {}
    /// Loop function for task for test_two_infinite_tasks_task_manager.
    fn test_two_infinite_tasks_task_manager_loop_fn2() {
        TEST_TWO_INFINITE_TASK_TASK_MANAGER_COUNTER2.fetch_add(1, Ordering::Relaxed);
    }
    /// Stop function for a task for test_two_infinite_tasks_task_manager.
    fn test_two_infinite_tasks_task_manager_stop_condition_fn2() -> bool {
        false
    }
    #[test]
    #[sequential]
    /// Tests if task manager with two infinite tasks works correctly during some without panic.
    fn test_two_infinite_tasks_task_manager() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(
            test_two_infinite_tasks_task_manager_setup_fn1,
            test_two_infinite_tasks_task_manager_loop_fn1,
            test_two_infinite_tasks_task_manager_stop_condition_fn1,
        );
        TaskManager::add_task(
            test_two_infinite_tasks_task_manager_setup_fn2,
            test_two_infinite_tasks_task_manager_loop_fn2,
            test_two_infinite_tasks_task_manager_stop_condition_fn2,
        );
        TaskManager::test_start_task_manager();
    }

    /// Counter for a task for test_setup_task_manager.
    static TEST_SETUP_TASK_MANAGER_COUNTER: AtomicU32 = AtomicU32::new(1);
    /// Setup function for task for test_setup_task_manager.
    fn test_setup_task_manager_setup_fn() {
        TEST_SETUP_TASK_MANAGER_COUNTER.store(42, Ordering::Relaxed);
    }
    /// Loop function for task for test_setup_task_manager.
    fn test_setup_task_manager_loop_fn() {}
    /// Stop function for task for test_setup_task_manager.
    fn test_setup_task_manager_stop_condition_fn() -> bool {
        false
    }
    #[test]
    #[sequential]
    /// Tests if task manager works correctly with setup function during some time without panic.
    fn test_setup_task_manager() {
        TaskManager::reset_task_manager();
        TaskManager::add_task(
            test_setup_task_manager_setup_fn,
            test_setup_task_manager_loop_fn,
            test_setup_task_manager_stop_condition_fn,
        );
        TaskManager::test_start_task_manager();

        assert_eq!(
            unsafe { TEST_SETUP_TASK_MANAGER_COUNTER.as_ptr().read() },
            42
        );
    }

    #[test]
    /// Tests setup timer function and getting counter value (bad unit test).
    fn test_setup_timer() {
        Timer::setup_timer();
        let timer = Timer::get_timer(0)
            .expect("The timer is already active or a timer with this index does not exist.");
        assert_eq!(timer.get_time().as_micros(), 0);
        timer.release_timer();
    }

    #[test]
    /// Tests loop timer function.
    fn test_loop_timer() {
        Timer::setup_timer();
        let mut timer = Timer::get_timer(0)
            .expect("The timer is already active or a timer with this index does not exist.");
        timer.loop_timer();
        assert_eq!(timer.get_time().as_micros(), 0);
        timer.release_timer();
    }

    #[test]
    /// Tests stop condition timer function.
    fn test_stop_condition_timer() {
        let timer = Timer::get_timer(0)
            .expect("The timer is already active or a timer with this index does not exist.");
        timer.change_period_timer(Duration::new(10, 0));
        timer.start_timer();
        assert!(!timer.stop_condition_timer());
        timer.release_timer();
    }
}
