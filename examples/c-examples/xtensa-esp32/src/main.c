#include <string.h>
#include <stdbool.h>

extern unsigned int _sbss, _ebss, _sidata, _sdata, _edata;
extern void add_task(void (*setup_fn)(), void (*loop_fn)(), bool (*stop_condition_fn)());
extern void start_task_manager();
extern void init_system();

int counter = 0;

void setup_fn() {
}

void loop_fn() {
    counter++;
}

bool stop_condition_fn() {
    if (counter == 50) {
        return true;
    }

    return false;
}

int main( void ) {
    printf("Start");
    init_system();
    add_task(setup_fn, loop_fn, stop_condition_fn);
    start_task_manager();
    printf("Success");

    // (Should never be reached)
    return 0;
}

// Application entry point / startup logic.
void __attribute__( ( noreturn ) ) call_start_cpu0() {
    // Clear BSS.
    memset( &_sbss, 0, ( &_ebss - &_sbss ) * sizeof( _sbss ) );
    // Copy initialized data.
    memmove( &_sdata, &_sidata, ( &_edata - &_sdata ) * sizeof( _sdata ) );

    // Done, branch to main
    main();
    // (Should never be reached)
    while( 1 ) {}
}
