#include <stdio.h>
extern void example_init_system(void);

void app_main(void) {
    example_init_system();
    printf("Hello world from C!\n");
}
