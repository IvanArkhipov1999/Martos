#include <stdio.h>

extern long get_tick_counter();

void app_main(void)
{
    long tick = get_tick_counter();
    printf("tick in c = %ld\n", tick);

    for (;;) {}
}
