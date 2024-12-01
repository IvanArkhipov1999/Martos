#[no_mangle]
pub extern "C" fn start_task_manager()
{
    martos::c_api::start_task_manager();
}

#[no_mangle]
pub extern "C" fn init_system()
{
    martos::init_system();
}