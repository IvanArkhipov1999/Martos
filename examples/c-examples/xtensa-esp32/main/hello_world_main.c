#include <stdio.h>
#include <stdbool.h>

extern long get_tick_counter();

extern void add_task(void (*setup_fn)(), void (*loop_fn)(), bool (*stop_condition_fn)());
extern void start_task_manager();

int counter = 0;

void setup_fn() {
    printf("Setup hello world!\n");
}

void loop_fn() {
    counter++;
    printf("Loop hello world!\n");
    printf("counter = %i\n", counter);
}

bool stop_condition_fn() {
    if (counter == 50) {
        return true;
    }

    return false;
}

void app_main(void)
{
    add_task(setup_fn, loop_fn, stop_condition_fn);
    start_task_manager();
}
