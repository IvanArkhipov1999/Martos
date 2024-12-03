extern "C"
{
    pub fn init_system();
    pub fn start_task_manager();
}

#[no_mangle]
extern "C" fn example_init_system()
{
    unsafe {
        init_system();
    }
}

#[no_mangle]
extern "C" fn example_start_task_manager()
{
    unsafe {
        start_task_manager();
    }
}