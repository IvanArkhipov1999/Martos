#include <string.h>
#include <stdbool.h>
extern unsigned int _bss_start, _bss_end, _sidata, _data_start, _data_end;

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
    init_system();
    add_task(setup_fn, loop_fn, stop_condition_fn);
    start_task_manager();

    // (Should never be reached)
    return 0;
}

// Application entry point / startup logic.
void __attribute__( ( noreturn ) ) call_start_cpu0() {
    // Clear BSS.
    memset( &_bss_start, 0, ( &_bss_end - &_bss_start ) * sizeof( _bss_start ) );
    // Copy initialized data.
    memmove( &_data_start, &_sidata, ( &_data_end - &_data_start ) * sizeof( _data_start ) );

    // Done, branch to main
    main();
    // (Should never be reached)
    while( 1 ) {}
}
